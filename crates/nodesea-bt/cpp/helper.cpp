#include "nodesea_bt/helper.hpp"

#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>

bool is_port_available(uint16_t port) {
  // Create a UDP socket to probe local port availability
  int sock = ::socket(AF_INET, SOCK_DGRAM, 0);
  if (sock < 0) {
    return false;
  }

  sockaddr_in addr{};
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = INADDR_ANY;
  addr.sin_port = htons(port);

  int res = ::bind(sock, reinterpret_cast<const struct sockaddr *>(&addr), sizeof(addr));
  ::close(sock);

  return res == 0;
}
