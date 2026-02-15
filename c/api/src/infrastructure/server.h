#ifndef SERVER_H
#define SERVER_H

int  create_and_bind_socket(int port);
void cleanup_server(int sockfd);

#endif
