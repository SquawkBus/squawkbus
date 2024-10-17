#ifndef SQUAWKBUS_TOPICBUS_SERVER_INTERACTOR_HPP
#define SQUAWKBUS_TOPICBUS_SERVER_INTERACTOR_HPP

#include <memory>
#include <vector>

#include "io/poller.hpp"
#include "serialization/frame_reader.hpp"

#include "topicbus/messages/message.hpp"

namespace squawkbus::topicbus
{
  using squawkbus::io::Poller;
  using squawkbus::serialization::FrameReader;
  using squawkbus::topicbus::messages::Message;

  class Interactor
  {
  private:
    int fd_;
    Poller& poller_;
    FrameReader reader_;

  public:
    Interactor(int fd, Poller& poller);

    int fd() const noexcept { return fd_; }

    void receive(std::vector<std::vector<char>>&& bufs);
    void send(std::shared_ptr<Message> message);

  private:
    void process_message(std::shared_ptr<Message> message);
  };

}

#endif // SQUAWKBUS_TOPICBUS_SERVER_INTERACTOR_HPP
