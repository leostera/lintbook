# Test fixtures for PY024 (F707) - default-except-not-last

# Bad: Default except not last (should trigger PY024)
try:
    risky_operation()
except:  # Bare except should be last
    handle_general_error()
except ValueError:  # Specific except after bare except - wrong order
    handle_value_error()

try:
    another_operation()
except Exception:  # General exception should be last
    handle_exception()
except TypeError:  # Specific except after general exception - wrong order
    handle_type_error()

# Bad: Multiple specific exceptions after bare except
try:
    complex_operation()
except:  # Bare except should be last
    handle_any_error()
except (ValueError, TypeError):  # Specific exceptions after bare except
    handle_specific_errors()
except KeyError:  # Another specific except after bare except
    handle_key_error()

# Bad: Exception after bare except
try:
    operation()
except:  # Bare except should be last
    log_error()
except Exception as e:  # Exception after bare except
    handle_exception(e)

# Bad: More specific exceptions after less specific ones
try:
    file_operation()
except Exception:  # General exception
    handle_general()
except IOError:  # More specific than Exception - wrong order
    handle_io_error()
except FileNotFoundError:  # Even more specific - wrong order
    handle_file_not_found()

# Good: Correct order - specific to general
try:
    risky_operation()
except ValueError:  # Most specific first
    handle_value_error()
except TypeError:  # Another specific exception
    handle_type_error()
except Exception:  # General exception
    handle_exception()
except:  # Bare except last (if used at all)
    handle_any_error()

try:
    file_operation()
except FileNotFoundError:  # Most specific
    handle_file_not_found()
except IOError:  # Less specific than FileNotFoundError
    handle_io_error()
except Exception:  # General exception
    handle_general()

# Good: Only specific exceptions
try:
    specific_operation()
except ValueError:
    handle_value_error()
except TypeError:
    handle_type_error()
except KeyError:
    handle_key_error()

# Good: Only bare except (though not recommended)
try:
    simple_operation()
except:
    handle_any_error()

# Good: Only Exception
try:
    operation()
except Exception as e:
    handle_exception(e)

# Good: Multiple specific exceptions with tuple
try:
    operation()
except (ValueError, TypeError) as e:
    handle_multiple_types(e)
except KeyError:
    handle_key_error()

# Good: Nested try-except (each block independent)
try:
    outer_operation()
    try:
        inner_operation()
    except ValueError:  # This is fine - inner try block
        handle_inner_value_error()
    except:  # This is fine - bare except last in inner block
        handle_inner_any_error()
except TypeError:  # This is fine - outer try block
    handle_outer_type_error()

# Bad: Mixed order in complex scenario
try:
    complex_scenario()
except (ValueError, TypeError):  # Specific exceptions
    handle_multiple()
except:  # Bare except - should be last
    handle_any()
except Exception:  # Exception after bare except - wrong
    handle_exception()
except RuntimeError:  # Specific after bare except - wrong
    handle_runtime_error()

# Edge case: finally clause (shouldn't affect ordering)
try:
    operation_with_cleanup()
except ValueError:
    handle_value_error()
except:  # Bare except should still be last
    handle_any_error()
except Exception:  # Exception after bare except - wrong
    handle_exception()
finally:
    cleanup()

# Good: Inheritance hierarchy respected
try:
    inheritance_test()
except KeyError:  # Most specific
    handle_key_error()
except LookupError:  # Parent of KeyError
    handle_lookup_error()
except Exception:  # Top-level exception
    handle_exception()

# Bad: Wrong inheritance order
try:
    inheritance_test()
except LookupError:  # Parent class first - wrong
    handle_lookup_error()
except KeyError:  # Child class after parent - should be first
    handle_key_error()

# Good: Custom exceptions with proper ordering
class CustomError(Exception):
    pass

class SpecificCustomError(CustomError):
    pass

try:
    custom_operation()
except SpecificCustomError:  # Most specific first
    handle_specific_custom()
except CustomError:  # Less specific
    handle_custom()
except Exception:  # General
    handle_general()

# Bad: Custom exceptions with wrong ordering
try:
    custom_operation()
except CustomError:  # Parent class first - wrong
    handle_custom()
except SpecificCustomError:  # Child class should be first
    handle_specific_custom()

# Good: As clauses don't affect ordering
try:
    operation()
except ValueError as ve:
    handle_value_error(ve)
except Exception as e:
    handle_exception(e)

# Multiple try-except blocks (each independent)
try:
    first_operation()
except ValueError:
    handle_first_value_error()
except:
    handle_first_any_error()

try:
    second_operation()
except TypeError:
    handle_second_type_error()
except Exception:
    handle_second_exception()

# Bad: Unreachable except clauses due to ordering
try:
    unreachable_test()
except BaseException:  # Too broad - catches everything
    handle_base()
except Exception:  # Unreachable - Exception is subclass of BaseException
    handle_exception()
except ValueError:  # Unreachable - ValueError is subclass of Exception
    handle_value_error()