def main():
    print("Starting...")
    
    # This should trigger the no-try-catch lint
    try:
        risky_operation()
    except Exception as e:
        print(f"Error: {e}")
    
    print("Done!")

def risky_operation():
    raise ValueError("Something went wrong")

if __name__ == "__main__":
    main()