# Test fixtures for PY032 (E0704) - misplaced-bare-raise

# Bad: Bare raise at module level
raise  # Wrong: bare raise outside exception handler

# Bad: Bare raise in function without exception context
def bad_function():
    raise  # Wrong: bare raise outside exception handler

# Bad: Bare raise in if statement (no exception context)
if condition:
    raise  # Wrong: bare raise outside exception handler

# Bad: Bare raise in loop (no exception context)
for item in items:
    raise  # Wrong: bare raise outside exception handler

# Bad: Bare raise in while loop (no exception context)
while condition:
    raise  # Wrong: bare raise outside exception handler

# Bad: Bare raise in finally block without exception context
try:
    some_operation()
finally:
    raise  # Wrong: bare raise in finally without active exception

# Bad: Bare raise in else block of try statement (no exception)
try:
    safe_operation()
except ValueError:
    handle_error()
else:
    raise  # Wrong: bare raise in else block (no exception context)

# Bad: Bare raise in nested function without exception context
def outer():
    def inner():
        raise  # Wrong: bare raise in nested function without context
    return inner

# Bad: Bare raise in class method without exception context
class BadClass:
    def method(self):
        raise  # Wrong: bare raise in method without exception context

# Bad: Bare raise in generator function without exception context
def bad_generator():
    yield 1
    raise  # Wrong: bare raise in generator without exception context

# Bad: Bare raise in async function without exception context
async def bad_async():
    raise  # Wrong: bare raise in async function without exception context

# Bad: Bare raise in comprehension (if possible)
# This is likely a syntax error, but test anyway
# bad_list = [raise for x in range(1)]  # Would be syntax error

# Bad: Bare raise in lambda (would be syntax error)
# bad_lambda = lambda: raise  # Syntax error

# Bad: Bare raise in decorator function without exception context
def bad_decorator(func):
    raise  # Wrong: bare raise in decorator without exception context
    return func

# Bad: Bare raise in context manager without exception context
class BadContextManager:
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        raise  # Wrong: bare raise without checking if there's an active exception

# Good: Bare raise in except clause (re-raising caught exception)
try:
    risky_operation()
except ValueError:
    log_error("ValueError occurred")
    raise  # Good: re-raising caught exception

# Good: Bare raise in nested except clause
try:
    dangerous_operation()
except Exception as e:
    try:
        cleanup_operation()
    except CleanupError:
        log_error("Cleanup failed")
        raise  # Good: re-raising cleanup exception
    raise  # Good: re-raising original exception

# Good: Bare raise in except clause with multiple exception types
try:
    complex_operation()
except (ValueError, TypeError) as e:
    handle_error(e)
    raise  # Good: re-raising caught exception

# Good: Bare raise in except clause after some processing
try:
    process_data()
except DataError as e:
    send_alert(f"Data processing failed: {e}")
    update_metrics("error_count")
    raise  # Good: re-raising after logging/metrics

# Good: Bare raise in nested try/except within except handler
def process_with_cleanup():
    try:
        main_operation()
    except MainError:
        try:
            emergency_cleanup()
        except CleanupError:
            raise  # Good: re-raising cleanup error
        raise  # Good: re-raising main error

# Good: Conditional bare raise in except clause
try:
    operation_that_might_fail()
except ConfigError as e:
    if e.severity == "critical":
        notify_admin(e)
        raise  # Good: conditionally re-raising based on error severity

# Good: Bare raise in except clause inside function
def error_handler():
    try:
        some_operation()
    except Exception:
        log_exception()
        raise  # Good: re-raising in function's except clause

# Good: Bare raise in except clause inside class method
class ErrorProcessor:
    def handle_error(self):
        try:
            self.process()
        except ProcessingError:
            self.cleanup()
            raise  # Good: re-raising in method's except clause

# Good: Bare raise in except clause inside async function
async def async_error_handler():
    try:
        await async_operation()
    except AsyncError:
        await async_cleanup()
        raise  # Good: re-raising in async except clause

# Good: Multiple except clauses with bare raises
try:
    multi_step_operation()
except ValidationError:
    log_validation_error()
    raise  # Good: re-raising validation error
except ProcessingError:
    log_processing_error()
    raise  # Good: re-raising processing error
except Exception:
    log_unknown_error()
    raise  # Good: re-raising unknown error

# Bad: Bare raise after exception handling is complete
try:
    operation()
except Exception:
    handle_exception()

# This is outside the exception handler
raise  # Wrong: bare raise after try/except block is complete

# Good: Bare raise in except clause with finally
try:
    operation_with_cleanup()
except Exception:
    handle_error()
    raise  # Good: re-raising before finally
finally:
    cleanup()

# Bad: Bare raise in finally after except (no active exception to re-raise)
try:
    operation()
except Exception:
    handle_error()
    # Exception is handled, not re-raised
finally:
    # raise  # Would be wrong: no active exception to re-raise
    pass

# Good: Function that always re-raises after logging
def logged_operation():
    try:
        return unsafe_operation()
    except Exception as e:
        logger.exception("Operation failed: %s", e)
        raise  # Good: always re-raise after logging

# Bad: Bare raise in different scope from exception handler
def exception_context():
    try:
        operation()
    except Exception:
        other_function()  # If other_function has bare raise, it's wrong

def other_function():
    raise  # Wrong: no exception context in this function

# Good: Proper exception chaining instead of bare raise when appropriate
try:
    operation()
except OriginalError as e:
    # This is an alternative to bare raise - explicit re-raising
    # raise e  # Also good: explicit re-raise
    raise  # Good: bare re-raise preserves traceback better

# Edge case: Bare raise in generator with exception context
def generator_with_exception():
    try:
        yield 1
        risky_operation()
        yield 2
    except Exception:
        yield -1
        raise  # Good: re-raising in generator's except clause