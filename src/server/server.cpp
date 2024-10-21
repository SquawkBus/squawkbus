#include <signal.h>

#include <cstdio>
#include <format>
#include <set>

#include "io/poller.hpp"
#include "io/tcp_listener_poll_handler.hpp"
#include "io/ssl_ctx.hpp"
#include "logging/log.hpp"
#include "utils/utils.hpp"

#include "popl.hpp"

#include "distributor.hpp"

using namespace squawkbus::io;
namespace logging = squawkbus::logging;
using squawkbus::server::Distributor;

std::shared_ptr<SslContext> make_ssl_context(const std::string& certfile, const std::string& keyfile)
{
  logging::info("making ssl server context");
  auto ctx = std::make_shared<SslServerContext>();
  ctx->min_proto_version(TLS1_2_VERSION);
  logging::info(std::format("Adding certificate file \"{}\"", certfile));
  ctx->use_certificate_file(certfile);
  logging::info(std::format("Adding key file \"{}\"", keyfile));
  ctx->use_private_key_file(keyfile);
  return ctx;
}

void echo_server(const std::string& host, std::uint16_t port, std::optional<std::shared_ptr<SslContext>> ssl_ctx)
{
  auto poll_client = std::make_shared<Distributor>();
  auto poller = Poller(poll_client);
  poller.add_handler(
    std::make_unique<TcpListenerPollHandler>(port, ssl_ctx),
    host,
    port);
  poller.event_loop();
}

int main(int argc, char** argv)
{
  // signal(SIGPIPE,SIG_IGN);

  bool use_tls = false;
  uint16_t port = 22000;
  popl::OptionParser op("options");
  op.add<popl::Switch>("s", "ssl", "Connect with TLS", &use_tls);
  auto help_option = op.add<popl::Switch>("", "help", "produce help message");
  op.add<popl::Value<decltype(port)>>("p", "port", "port number", port, &port);
  auto certfile_option = op.add<popl::Value<std::string>>("c", "certfile", "path to certificate file");
  auto keyfile_option = op.add<popl::Value<std::string>>("k", "keyfile", "path to key file");

  try
  {
    op.parse(argc, argv);

    if (help_option->is_set())
    {
      if (help_option->count() == 1)
        print_line(stderr, op.help());
	    else if (help_option->count() == 2)
		    print_line(stderr, op.help(popl::Attribute::advanced));
	    else
		    print_line(stderr, op.help(popl::Attribute::expert));
      exit(1);
    }

    logging::info(
      std::format(
        "starting echo server on port {}{}.",
        static_cast<int>(port),
        (use_tls ? " with TLS" : "")));

    std::optional<std::shared_ptr<SslContext>> ssl_ctx;

    if (use_tls)
    {
      if (!certfile_option->is_set())
      {
        print_line(stderr, "For ssl must use certfile");
        print_line(stderr, op.help());
        exit(1);
      }
      if (!keyfile_option->is_set())
      {
        print_line(stderr, "For ssl must use keyfile");
        print_line(stderr, op.help());
        exit(1);
      }
      ssl_ctx = make_ssl_context(certfile_option->value(), keyfile_option->value());
    }

    echo_server("localhost", port, std::move(ssl_ctx));
  }
  catch(const std::exception& error)
  {
    logging::error(std::format("Server failed: {}", error.what()));
  }
  catch (...)
  {
    logging::error(std::format("unknown error"));
  }

  logging::info("server stopped");


  return 0;
}