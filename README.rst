strecklistan
============

This is the backend of strecklistan: a simple web-shop.

It is entirely built in Rust_. The backend uses the Rocket_
web server framework. Database integration and migrations
are provided by Diesel_.

The frontend is built in the Seed_ framework.

.. _Rust:   https://www.rust-lang.org/
.. _Rocket: https://rocket.rs/
.. _Diesel: https://diesel.rs/
.. _Seed:   https://seed-rs.org/


Local Development
-----------------

Install the latest version of Rust using rustup_.

.. _rustup: https://rustup.rs/

The backend and the frontend need to be individually compiled.
See the guides for each.

Backend
^^^^^^^

All commands in this section are to be run in the ``backend``-folder.

You'll need the Postgres C/C++ client library ``libpq``, as well as
``openssl``. Install these via your preferred package manager.

You will also need a PostgreSQL database to connect to. Make sure
that you have one running. For a quick setup you can use docker. ::

    docker run --name "postgres" -d \
        --publish 5432:5432 \
        --env POSTGRES_PASSWORD=password \
        --env POSTGRES_USER=strecklistan \
        postgres:13

Then make sure to setup your local ``.env``-file with the proper
secrets and database settings. ::

    cp example.env .env

    $EDITOR .env

For handling migrations you need to use the ``diesel`` CLI. ::

    cargo install diesel_cli --no-default-features --features "postgres"

    diesel setup # Create database

    diesel migration run # Run migrations, generate rust bindings

You can then run the application using cargo. ::

    # Compile and run
    cargo run

    # Run tests
    cargo test

    # Automatically recompile and run on file changes
    cargo install -f cargo-watch
    cargo watch -x run

The backend server will serve the files for the frontend.
Make sure these are also built if you want to try the app.


Frontend
^^^^^^^^

All commands in this section are to be run in the ``frontend``-folder.

You'll need ``cargo-make``, and the webassembly compiler target.

Quick setup guide: ::

    # Install the webassembly toolchain
    rustup target add wasm32-unknown-unknown

    # Install cargo-make
    cargo install -f cargo-make

    # Compile the application
    cargo make build

    # Or: Automatically recompile on file changes
    cargo make watch

