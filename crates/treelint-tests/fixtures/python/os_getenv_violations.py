#!/usr/bin/env python

import os

# These should trigger PY003
port = os.getenv('PORT')
host = os.getenv('HOST', 'localhost')
db_url = os.getenv('DATABASE_URL')

# This is the preferred approach
import config
postgres_host = config.postgres_host()
postgres_port = config.postgres_port()