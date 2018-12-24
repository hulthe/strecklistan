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

Install Rust using rustup_. This application currently requires Rust version
1.33, which means that you have to install the nightly branch (as of the time
when writing this).

.. _rustup: https://rustup.rs/

You also require the Postgres C/C++ client library: ``libpq``.
Install this via your preferred package manager,
otherwise compiling will fail with linking errors.

Then make sure to setup your local ``.env``-file. ::

    cp example.env .env

    $EDITOR .env

For handling migrations you need to use the ``diesel`` CLI. ::

    cargo install diesel_cli --no-default-features --features "postgres"

    diesel setup # Create database

    diesel migration run # Run migrations, generate rust bindings

You can then build the application using cargo. ::

    cargo run # Or cargo build if you just want the binary
