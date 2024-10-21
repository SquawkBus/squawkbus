#ifndef SQUAWKBUS_CLIENT_TOPIC_CLIENT_HPP
#define SQUAWKBUS_CLIENT_TOPIC_CLIENT_HPP

#include <cstddef>
#include <memory>
#include <string>
#include <variant>

#include "io/poller.hpp"
#include "io/tcp_client_socket.hpp"

namespace squawkbus::client
{
  using squawkbus::io::Poller;
  using squawkbus::io::PollClient;
  using squawkbus::io::TcpClientSocket;

  class TopicClient : public PollClient
  {
  private:
    std::shared_ptr<TcpClientSocket> client_socket_;

  public:
    TopicClient(std::shared_ptr<TcpClientSocket> client_socket);

    void on_startup(Poller& poller) override;
    void on_open(Poller& poller, int fd, const std::string& host, std::uint16_t port) override;
    void on_close(Poller& poller, int fd) override;
    void on_read(Poller& poller, int fd, std::vector<std::vector<char>>&& bufs) override;
    void on_error(Poller& poller, int fd, std::exception error) override;
  };

}

#endif // SQUAWKBUS_CLIENT_TOPIC_CLIENT_HPP