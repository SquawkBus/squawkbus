#ifndef SQUAWKBUS_MESSAGES_MESSAGE_TYPE_HPP
#define SQUAWKBUS_MESSAGES_MESSAGE_TYPE_HPP

#include <iostream>

#include "serialization/frame_buffer.hpp"
#include "serialization/frame_buffer_io.hpp"

namespace squawkbus::messages
{
  enum class MessageType : char
  {
    AuthenticationRequest        = 1,
    AuthenticationResponse       = 2,
    MulticastData                = 3,
    UnicastData                  = 4,
    ForwardedSubscriptionRequest = 5,
    NotificationRequest          = 6,
    SubscriptionRequest          = 7,
    ForwardedMulticastData       = 8,
    ForwardedUnicastData         = 9
  };

  inline std::string to_string(const MessageType& message_type)
  {
    switch (message_type)
    {
    case MessageType::AuthenticationRequest:
      return "AuthenticationRequest";
    case MessageType::AuthenticationResponse:
      return "AuthenticationResponse";
    case MessageType::MulticastData:
      return "MulticastData";
    case MessageType::UnicastData:
      return "UnicastData";
    case MessageType::ForwardedSubscriptionRequest:
      return "ForwardedSubscriptionRequest";
    case MessageType::NotificationRequest:
      return "NotificationRequest";
    case MessageType::SubscriptionRequest:
      return "SubscriptionRequest";
    case MessageType::ForwardedMulticastData:
      return "ForwardedMulticastData";
    case MessageType::ForwardedUnicastData:
      return "ForwardedUnicastData";
    default:
      return "unknown";
    }
  }

  inline std::ostream& operator << (std::ostream& os, MessageType message_type)
  {
    return os << to_string(message_type);
  }

  inline serialization::FrameBuffer& operator << (serialization::FrameBuffer& frame, MessageType t)
  {
    return frame << static_cast<char>(t);
  }

  inline serialization::FrameBuffer& operator >> (serialization::FrameBuffer& frame, MessageType& t)
  {
    char c;
    frame >> c;
    t = static_cast<MessageType>(c);
    return frame;    
  }
}

#endif // SQUAWKBUS_MESSAGES_MESSAGE_TYPE_HPP
