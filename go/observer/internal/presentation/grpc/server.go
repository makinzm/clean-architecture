package grpc

import (
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
	"makinzm/cleanarchitecture/gaze/internal/usecase"
	pb "makinzm/cleanarchitecture/gaze/proto/gazev1"
)

// Server implements the Gazer gRPC service.
type Server struct {
	pb.UnimplementedGazerServer
	snapshot *usecase.SortedSnapshot
	poller   repository.MetricRepository
}

// NewServer creates a gRPC server backed by the given repository.
func NewServer(repo repository.MetricRepository) *Server {
	return &Server{
		snapshot: usecase.NewSortedSnapshot(repo),
		poller:   repo,
	}
}

// Watch streams MetricEvents to the client.
func (s *Server) Watch(req *pb.WatchRequest, stream pb.Gazer_WatchServer) error {
	ctx := stream.Context()

	interval := 2 * time.Second
	if req.IntervalMs > 0 {
		interval = time.Duration(req.IntervalMs) * time.Millisecond
	}
	topN := int(req.TopN)

	sortKey := repository.SortByPID
	switch req.SortBy {
	case pb.SortBy_SORT_BY_CPU:
		sortKey = repository.SortByCPU
	case pb.SortBy_SORT_BY_MEM:
		sortKey = repository.SortByMem
	}

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return status.FromContextError(ctx.Err()).Err()
		case <-ticker.C:
			metrics, err := s.snapshot.TopN(ctx, sortKey, topN)
			if err != nil {
				return status.Errorf(codes.Internal, "fetch metrics: %v", err)
			}
			for _, m := range metrics {
				evt := &pb.MetricEvent{
					Pid:         int32(m.PID),
					Name:        m.Name,
					CpuPercent:  m.CPUPercent,
					MemBytes:    m.MemBytes,
					TimestampMs: m.Timestamp.UnixMilli(),
				}
				if err := stream.Send(evt); err != nil {
					return err
				}
			}
		}
	}
}
