laggit_backend
==============

This is the backend for the LaggIT web page.

It is built in Rust_ using the Rocket_ framework.
Database integration and migrations are provided by Diesel_.

.. _Rust: https://www.rust-lang.org/
.. _Rocket: https://rocket.rs/
.. _Diesel: https://diesel.rs/


Local Development
-----------------

Install the latest version of Rust using rustup_.

.. _rustup: https://rustup.rs/

You also require the Postgres C/C++ client library
``libpq``, as well as ``openssl``.
Install these via your preferred package manager.

Then make sure to setup your local ``.env``-file. ::

    cp example.env .env

    $EDITOR .env

For handling migrations you need to use the ``diesel`` CLI. ::

    cargo install diesel_cli --no-default-features --features "postgres"

    diesel setup # Create database

    diesel migration run # Run migrations, generate rust bindings

You can then build the application using cargo. ::

    cargo run # Or cargo build if you just want the binary
