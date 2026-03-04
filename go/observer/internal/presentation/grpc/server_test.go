package grpc_test

import (
	"context"
	"io"
	"net"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"makinzm/cleanarchitecture/gaze/internal/domain/entity"
	"makinzm/cleanarchitecture/gaze/internal/domain/repository"
	infragrpc "makinzm/cleanarchitecture/gaze/internal/presentation/grpc"
	pb "makinzm/cleanarchitecture/gaze/proto/gazev1"
)

// lib/bufconn.go provides a way to test gRPC without a real network.
const bufSize = 1024 * 1024

var lis *bufconn.Listener

// mockRepo for gRPC tests.
type mockRepo struct {
	metrics []entity.Metric
}

func (r *mockRepo) FetchAll(_ context.Context) ([]entity.Metric, error) {
	return r.metrics, nil
}

func (r *mockRepo) FetchSorted(_ context.Context, by repository.SortKey) ([]entity.Metric, error) {
	// Simple mock sort (CPU only for test)
	if by == repository.SortByCPU {
		return []entity.Metric{
			{PID: 2, Name: "high", CPUPercent: 90, Timestamp: time.Now()},
			{PID: 1, Name: "low", CPUPercent: 10, Timestamp: time.Now()},
		}, nil
	}
	return r.metrics, nil
}

func (r *mockRepo) Stream(_ context.Context) (<-chan entity.Metric, error) {
	ch := make(chan entity.Metric)
	close(ch)
	return ch, nil
}

func initGRPCServer(repo repository.MetricRepository) *grpc.Server {
	lis = bufconn.Listen(bufSize)
	s := grpc.NewServer()
	pb.RegisterGazerServer(s, infragrpc.NewServer(repo))
	go func() {
		if err := s.Serve(lis); err != nil {
			panic(err)
		}
	}()
	return s
}

func bufDialer(context.Context, string) (net.Conn, error) {
	return lis.Dial()
}

func TestWatchStreaming(t *testing.T) {
	repo := &mockRepo{metrics: []entity.Metric{
		{PID: 1, Name: "test", CPUPercent: 10, Timestamp: time.Now()},
	}}
	s := initGRPCServer(repo)
	defer s.Stop()

	ctx := context.Background()
	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(bufDialer), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("failed to dial bufnet: %v", err)
	}
	defer conn.Close()

	client := pb.NewGazerClient(conn)
	stream, err := client.Watch(ctx, &pb.WatchRequest{
		IntervalMs: 100, // fast tick
		TopN:       1,
		SortBy:     pb.SortBy_SORT_BY_CPU,
	})
	if err != nil {
		t.Fatalf("Watch failed: %v", err)
	}

	// Read first event
	evt, err := stream.Recv()
	if err == io.EOF {
		t.Fatal("unexpected EOF")
	}
	if err != nil {
		t.Fatalf("Recv failed: %v", err)
	}

	if evt.Pid != 2 {
		t.Errorf("expected PID 2 (high CPU), got %d", evt.Pid)
	}
	if evt.Name != "high" {
		t.Errorf("expected 'high', got %q", evt.Name)
	}
}
