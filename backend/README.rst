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
1.30, which means that you have to install the nightly branch (as of the time
when writing this).

.. _rustup: https://rustup.rs/

Then make sure to setup your local ``.env``-file. ::

    cp example.env .env

    $EDITOR .env

For handling migrations you need to use ``diesel_cli``. ::

    cargo install diesel_cli

    diesel_cli setup

You can then build the application using cargo. ::

    cargo run
