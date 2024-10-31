#ifndef SQUAWKBUS_SERVER_SERVER_HPP
#define SQUAWKBUS_SERVER_SERVER_HPP

#include <filesystem>
#include <map>
#include <optional>
#include <string>
#include <memory>
#include <vector>

#include "io/poller.hpp"

#include "authorization.hpp"
#include "interactor.hpp"
#include "hub.hpp"

namespace squawkbus::server
{
  using io::PollClient;
  using io::Poller;
  
  class Distributor : public PollClient
  {
  private:
    std::map<int, std::shared_ptr<Interactor>> interactors_;
    Hub hub_;

  public:
    Distributor(AuthorizationManager&& authorization_manager)
      : hub_(std::move(authorization_manager))
    {
    }

  private:
    // The implementation of PollClient
    void on_startup(Poller& poller) override;
    void on_interrupt(Poller& poller) override;
    void on_open(Poller& poller, int fd, const std::string& host, std::uint16_t port) override;
    void on_close(Poller& poller, int fd) override;
    void on_read(Poller& poller, int fd, std::vector<std::vector<char>>&& bufs) override;
    void on_error(Poller& poller, int fd, std::exception error) override;

  };
}

#endif // SQUAWKBUS_SERVER_SERVER_HPP
