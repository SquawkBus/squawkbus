#ifndef SQUAWKBUS_MESSAGES_FORWARDED_SUBSCRIPTION_REQUEST_HPP
#define SQUAWKBUS_MESSAGES_FORWARDED_SUBSCRIPTION_REQUEST_HPP

#include <format>
#include <memory>
#include <string>

#include "serialization/frame_buffer.hpp"
#include "serialization/frame_buffer_io.hpp"

#include "messages/_message_type.hpp"
#include "messages/_message.hpp"

namespace squawkbus::messages
{
  using serialization::FrameBuffer;

  class ForwardedSubscriptionRequest : public Message
  {
  public:
    std::string user;
    std::string host;
    std::string client_id;
    std::string topic;
    bool is_add;

  public:
    ForwardedSubscriptionRequest() noexcept
      : Message(MessageType::ForwardedSubscriptionRequest)
    {
    }

    ForwardedSubscriptionRequest(
      const std::string &user,
      const std::string &host,
      const std::string &client_id,
      const std::string &topic,
      bool is_add) noexcept
      : Message(MessageType::ForwardedSubscriptionRequest),
        user(user),
        host(host),
        client_id(client_id),
        topic(topic),
        is_add(is_add)
    {
    }

    bool operator==(const ForwardedSubscriptionRequest &other) const noexcept
    {
      return Message::operator==(other) &&
        user == other.user &&
        host == other.host &&
        client_id == other.client_id &&
        topic == other.topic &&
        is_add == other.is_add;
    }

    bool equals(const Message* other) const noexcept override
    {
      return operator==(*dynamic_cast<const ForwardedSubscriptionRequest*>(other));
    }

    std::string str() const override
    {
      return std::format(
        "ForwardedSubscriptionRequest(message_type={},user=\"{}\",host=\"{}\",client_id=\"{}\",topic=\"{}\",is_add={})",
        messages::to_string(message_type),
        user,
        host,
        client_id,
        topic,
        (is_add ? "<true>" : "<false>"));
    }

  protected:

    void serialize_body(FrameBuffer &frame) const override
    {
      frame
        << user
        << host
        << client_id
        << topic
        << is_add;
    }

    void deserialize_body(FrameBuffer &frame) override
    {
      frame
        >> user
        >> host
        >> client_id
        >> topic
        >> is_add;
    }
  };
}

#endif // SQUAWKBUS_MESSAGES_FORWARDED_SUBSCRIPTION_REQUEST_HPP
