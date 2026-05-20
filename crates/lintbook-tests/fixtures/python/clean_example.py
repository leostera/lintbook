def safe_function():
    """A function without try/except blocks."""
    result = 42 * 2
    return result

def main():
    value = safe_function()
    print(f"The answer is: {value}")
    
if __name__ == "__main__":
    main()