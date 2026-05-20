#!/usr/bin/env python

# This should trigger PY004
try:
    risky_operation()
except:
    print("Something went wrong")

# This is okay - specific exception
try:
    another_operation()
except ValueError:
    print("Invalid value")

# This should also trigger PY004
try:
    complex_operation()
except KeyError:
    print("Key not found")
except:
    print("Some other error")

# This is okay - catching Exception explicitly
try:
    final_operation()
except Exception as e:
    print(f"Error: {e}")