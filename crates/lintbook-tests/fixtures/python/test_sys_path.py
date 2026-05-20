#!/usr/bin/env python

import sys

# This should trigger PY002
sys.path.append('/some/custom/path')

# This should also trigger PY002  
sys.path.insert(0, '/another/path')

# This is okay - just reading
print(sys.path)

# This should trigger PY002
sys.path = []