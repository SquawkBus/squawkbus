#include "serialization/frame_buffer.hpp"
#include "messages/messages.hpp"

#define CATCH_CONFIG_MAIN
#include "catch2/catch.hpp"

using squawkbus::serialization::FrameBuffer;
using squawkbus::messages::DataPacket;

TEST_CASE("smoke test") {
    auto dp = DataPacket(
        static_cast<std::int32_t>(1),
        std::string("text/plain"),
        std::vector<char> { 'H', 'e', 'l', 'l', 'o' }
    );
    auto user_entitlements_pass = std::set<std::int32_t> { 1, 2, 3 };
    auto user_entitlements_fail = std::set<std::int32_t> { 4, 5, 6 };
    REQUIRE(dp.is_authorized(user_entitlements_pass));
    REQUIRE_FALSE(dp.is_authorized(user_entitlements_fail));
}

TEST_CASE("frames") {
    FrameBuffer frame;

    auto dp1 = DataPacket(
        static_cast<std::int32_t>(1),
        "text/plain",
        std::vector<char> { 'H', 'e', 'l', 'l', 'o' }
    );
    auto dp2 = DataPacket(
        static_cast<std::int32_t>(1),
        "text/plain",
        std::vector<char> { 'W', 'o', 'r', 'l', 'd' }
    );
    std::vector<DataPacket> data_packets_in, data_packets_out;
    data_packets_in.push_back(dp1);
    data_packets_in.push_back(dp2);

    frame << data_packets_in;
    frame >> data_packets_out;
    REQUIRE(data_packets_in == data_packets_out);
}