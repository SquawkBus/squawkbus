#include "authentication_manager.hpp"
#include "authentication_repository.hpp"

#include <format>
#include <fstream>
#include <map>
#include <stdexcept>
#include <string>
#include <utility>

#include "logging/log.hpp"

#include "serialization/frame_reader.hpp"
#include "serialization/frame_buffer.hpp"
#include "serialization/frame_buffer_io.hpp"

#include "messages/messages.hpp"

namespace squawkbus::server
{
  namespace
  {
    auto log = logging::logger("squawkbus");
  }

  using squawkbus::messages::AuthenticationRequest;
  using squawkbus::serialization::FrameBuffer;
  using squawkbus::serialization::FrameReader;

  void AuthenticationManager::load()
  {
    if (!password_file_)
      return;

    log.info(std::format("Loading password file {}", *password_file_));

    auto file = std::fstream(*password_file_, std::ios::in);
    if (!file.is_open())
      throw std::runtime_error("failed to open password file");

    std::map<std::string, std::string> entries;
    std::string line;
    while (std::getline(file, line))
    {
      // Skip comments.
      if (line.starts_with("#"))
        continue;

      // Split user and data.
      auto colon = line.find(':');
      if (colon == std::string::npos)
        throw std::runtime_error("invalid password record");

      auto user = line.substr(0, colon);
      auto data = line.substr(colon+1);

      entries.insert({user, data});
    }

    repository_ = AuthenticationRepository(std::move(entries));
  }

  std::optional<std::string> AuthenticationManager::authenticate(AuthenticationRequest&& message) const
  {
    log.debug(std::format("Authenticating \"{}\"", message.method));

    if (message.method == "NONE")
    {
      return authenticate_none(message);
    }
    else if (message.method == "HTPASSWD")
    {
      return authenticate_htpasswd(message);
    }
    else
    {
      return std::nullopt;
    }
  }

  std::optional<std::string> AuthenticationManager::authenticate_none(AuthenticationRequest& message) const
  {
      return "nobody";
  }

  std::optional<std::string> AuthenticationManager::authenticate_htpasswd(AuthenticationRequest& message) const
  {
    auto reader = FrameReader();
    reader.write(std::move(message.data));
    if (!reader.has_frame())
    {
      log.error("invalid authentication data");
      return std::nullopt;
    }
    auto frame = reader.read();
    std::string username, password;
    frame >> username >> password;
    if (!repository_.authenticate(username, password))
      return std::nullopt;

    return username;
  }

}