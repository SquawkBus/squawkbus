#ifndef SQUAWKBUS_FEEDBUS_MESSAGES_FORWARDED_UNICAST_DATA_HPP
#define SQUAWKBUS_FEEDBUS_MESSAGES_FORWARDED_UNICAST_DATA_HPP

#include <format>
#include <memory>
#include <string>
#include <vector>

#include "serialization/frame_buffer.hpp"
#include "serialization/frame_buffer_io.hpp"
#include "serialization/data_packet.hpp"

#include "feedbus/messages/message_type.hpp"
#include "feedbus/messages/message.hpp"

namespace squawkbus::feedbus::messages
{
    using serialization::FrameBuffer;
    using serialization::DataPacket;

    struct ForwardedUnicastData : public Message
    {
        std::string user;
        std::string host;
        std::string client_id;
        std::string feed;
        std::string topic;
        std::string content_type;
        std::vector<DataPacket> data_packets;

        ForwardedUnicastData()
            : Message(MessageType::ForwardedUnicastData)
        {
        }

        ForwardedUnicastData(
            const std::string &user,
            const std::string &host,
            const std::string &client_id,
            const std::string &feed,
            const std::string &topic,
            const std::string &content_type,
            const std::vector<DataPacket> &data_packets)
            :   Message(MessageType::ForwardedUnicastData),
                user(user),
                host(host),
                client_id(client_id),
                feed(feed),
                topic(topic),
                content_type(content_type),
                data_packets(data_packets)
        {
        }

        void write_body(FrameBuffer &frame) const override
        {
            frame
                << user
                << host
                << client_id
                << feed
                << topic
                << content_type
                << data_packets;
        }

        void read_body(FrameBuffer &frame) override
        {
            frame
                >> user
                >> host
                >> client_id
                >> feed
                >> topic
                >> content_type
                >> data_packets;
        }

        bool equals(const std::shared_ptr<ForwardedUnicastData> &other) const
        {
            return
                message_type == other->message_type &&
                user == other->user &&
                host == other->host &&
                client_id == other->client_id &&
                feed == other->feed &&
                topic == other->topic &&
                content_type == other->content_type &&
                data_packets == other->data_packets;
        }

        bool equals(const std::shared_ptr<Message>& other) const override
        {
            return equals(std::static_pointer_cast<ForwardedUnicastData>(other));
        }

        std::string to_string() const override
        {
            return std::format(
                "ForwardedUnicastData(message_type={},user=\"{}\",host=\"{}\",client_id=\"{}\",feed=\"{}\",topic=\"{}\",content_type=\"{}\",data_packets={})",
                messages::to_string(message_type),
                user,
                host,
                client_id,
                feed,
                topic,
                content_type,
                ::to_string(data_packets)
            );
        }
    };
}

#endif // SQUAWKBUS_FEEDBUS_MESSAGES_FORWARDED_UNICAST_DATA_HPP