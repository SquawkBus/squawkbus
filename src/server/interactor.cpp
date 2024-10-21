#include <format>
#include <stdexcept>
#include <string>

#include "logging/log.hpp"

#include "serialization/frame_buffer.hpp"
#include "serialization/frame_buffer_io.hpp"

#include "interactor.hpp"
#include "hub.hpp"

namespace squawkbus::server
{
  using squawkbus::io::Poller;
  using squawkbus::serialization::FrameBuffer;
  using squawkbus::messages::Message;
  using squawkbus::messages::MessageType;
  using squawkbus::messages::Authenticate;

  Interactor::Interactor(
    int fd,
    Poller& poller,
    Hub& hub,
    const std::string& host,
    std::uint16_t port)
    : fd_(fd),
      host_(host),
      id_(std::format("{}:{}", host, port)),
      poller_(poller),
      hub_(hub)
  {
  }

  void Interactor::send(std::shared_ptr<Message> message)
  {
    auto frame = message->serialize();
    auto buf = std::vector<char>(frame);
    poller_.write(fd_, buf);
  }

  void Interactor::receive(std::vector<char>&& buf)
  {
    reader_.write(buf);

    while (reader_.has_frame())
    {
      auto frame = reader_.read();
      auto message = Message::deserialize(frame);
      process_message(message.get());
    }
  }

  void Interactor::process_message(Message* message)
  {
    if (user_ == std::nullopt)
      authenticate(message);
    else
      hub_.on_message(this, message);
  }

  void Interactor::authenticate(Message* message)
  {
    if (message->message_type() != MessageType::Authenticate)
      throw std::runtime_error("expected authenticate message");

    auto authenticate_message = dynamic_cast<Authenticate*>(message);
    if (authenticate_message->method() == "PLAIN")
    {
      user_ = authenticate_message->data().empty()
        ? std::string("nobody")
        : std::string(
            authenticate_message->data().begin(),
            authenticate_message->data().end());
      logging::info(std::format("authenticated as {}", *user_));
      return;
    }

    throw std::runtime_error("unknown authentication method");
  }
}