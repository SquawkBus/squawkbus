#ifndef SQUAWKBUS_SERVER_INTERACTOR_HPP
#define SQUAWKBUS_SERVER_INTERACTOR_HPP

#include <cstddef>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "io/poller.hpp"
#include "serialization/frame_reader.hpp"

#include "messages/messages.hpp"

namespace squawkbus::server
{
  using squawkbus::io::Poller;
  using squawkbus::serialization::FrameReader;
  using squawkbus::messages::Message;

  class Hub;

  class Interactor
  {
  private:
    int fd_;
    std::string host_;
    std::string id_;
    Poller& poller_;
    Hub& hub_;
    FrameReader reader_;
    std::optional<std::string> user_;

  public:
    Interactor(int fd, Poller& poller, Hub& hub, const std::string& host, std::uint16_t port);

    int fd() const noexcept { return fd_; }
    const std::string& host() const noexcept { return host_; }

    void receive(std::vector<char>&& buf);
    void send(std::shared_ptr<Message> message);

  private:
    void process_message(Message* message);
    void authenticate(Message* message);
  };

}

#endif // SQUAWKBUS_SERVER_INTERACTOR_HPP