# Test fixtures for PY028 (E0116) - continue-in-finally

# Bad: Continue statement in finally block (should trigger PY028)
def bad_continue_in_finally():
    for i in range(10):
        try:
            process_item(i)
        except Exception:
            handle_exception()
        finally:
            if should_skip(i):
                continue  # Wrong: continue not allowed in finally block

# Bad: Continue in nested try-finally
def nested_continue_in_finally():
    for outer in range(5):
        try:
            for inner in range(3):
                try:
                    risky_operation(outer, inner)
                except ValueError:
                    pass
                finally:
                    if error_condition:
                        continue  # Wrong: continue in finally
        except Exception:
            pass

# Bad: Continue in finally with complex logic
def complex_finally_continue():
    for item in items:
        resource = None
        try:
            resource = acquire_resource()
            process(item, resource)
        except ResourceError:
            log_error("Resource error")
        except ProcessingError:
            log_error("Processing error")
        finally:
            if resource:
                try:
                    resource.cleanup()
                except CleanupError:
                    log_error("Cleanup failed")
                    if is_critical_error():
                        continue  # Wrong: continue in finally
            else:
                continue  # Wrong: continue in finally

# Bad: Multiple continues in finally
def multiple_continues_in_finally():
    for i in range(10):
        try:
            process(i)
        finally:
            if condition1:
                continue  # Wrong
            elif condition2:
                continue  # Wrong
            else:
                cleanup()

# Good: Continue in try block (allowed)
def good_continue_in_try():
    for i in range(10):
        try:
            if should_skip(i):
                continue  # Correct: continue in try block is allowed
            process_item(i)
        except Exception:
            handle_exception()
        finally:
            cleanup()

# Good: Continue in except block (allowed)
def good_continue_in_except():
    for i in range(10):
        try:
            risky_operation(i)
        except SkipException:
            continue  # Correct: continue in except block is allowed
        except CriticalException:
            break
        finally:
            cleanup()

# Good: Continue in loop without finally
def good_continue_no_finally():
    for i in range(10):
        try:
            if should_skip(i):
                continue  # Correct: no finally block
            process_item(i)
        except Exception:
            if can_recover():
                continue  # Correct: in except block
            else:
                break

# Good: Break in finally (different issue, but allowed in some contexts)
def break_in_finally():
    for i in range(10):
        try:
            process_item(i)
        finally:
            if critical_error():
                break  # This is break, not continue - different rule

# Good: Continue outside finally block
def continue_outside_finally():
    for i in range(10):
        if should_skip(i):
            continue  # Correct: not in finally block
        
        try:
            process_item(i)
        finally:
            cleanup()

# Good: Return in finally (allowed)
def return_in_finally():
    for i in range(10):
        try:
            result = process_item(i)
        finally:
            if should_return_early():
                return result  # Return is allowed in finally

# Good: Nested loops with proper continue placement
def nested_loops_proper_continue():
    for outer in range(5):
        for inner in range(3):
            try:
                if should_skip_inner(inner):
                    continue  # Correct: continue for inner loop, not in finally
                process(outer, inner)
            except Exception:
                if can_retry():
                    continue  # Correct: in except block
            finally:
                cleanup_inner()
        
        try:
            finalize_outer(outer)
        except Exception:
            if should_skip_outer():
                continue  # Correct: continue for outer loop, not in finally
        finally:
            cleanup_outer()

# Bad: Continue in finally of nested function
def function_with_nested_continue():
    for i in range(10):
        def nested_function():
            try:
                helper_operation()
            finally:
                if helper_failed():
                    continue  # Wrong: continue in finally (even in nested function)
        
        nested_function()

# Good: Finally block without continue
def proper_finally_usage():
    for i in range(10):
        resource = None
        try:
            resource = acquire_resource()
            result = process_item(i, resource)
            if not result.success:
                # Use return, break, or other control flow
                break  # or return, or raise exception
        except Exception as e:
            if can_skip_error(e):
                continue  # Correct: in except block
            else:
                raise
        finally:
            if resource:
                resource.cleanup()
            # No continue here - correct

# Edge case: Continue in finally of async function
async def async_continue_in_finally():
    for i in range(10):
        try:
            await async_process(i)
        finally:
            if should_skip():
                continue  # Wrong: continue in finally (async context)

# Edge case: Continue in finally with context manager
def continue_in_finally_with_context():
    for i in range(10):
        try:
            with context_manager() as ctx:
                ctx.process(i)
        finally:
            if cleanup_failed():
                continue  # Wrong: continue in finally

# Good: Continue with nested try-except (no finally)
def nested_try_no_finally():
    for i in range(10):
        try:
            try:
                risky_operation(i)
            except SpecificError:
                if can_continue():
                    continue  # Correct: no finally block in this try
                else:
                    raise
            process_item(i)
        except Exception:
            continue  # Correct: in except block

# Bad: Continue in finally with while loop
def continue_in_finally_while():
    i = 0
    while i < 10:
        try:
            process_item(i)
            i += 1
        finally:
            if error_occurred():
                continue  # Wrong: continue in finally (while loop context)

# Good: Proper error handling without continue in finally
def proper_error_handling():
    for i in range(10):
        success = False
        try:
            process_item(i)
            success = True
        except RecoverableError:
            continue  # Correct: in except block
        except CriticalError:
            break
        finally:
            if not success:
                log_failure(i)
            # No continue here - use other control flow mechanisms