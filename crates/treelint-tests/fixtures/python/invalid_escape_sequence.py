# Test fixtures for PY012 (W605) - invalid-escape-sequence

# Bad: Invalid escape sequences (should trigger PY012)
windows_path = "C:\Users\name\Documents\file.txt"
regex_pattern = "\d+\.\d+"
word_pattern = "\w+@\w+\.\w+"
latex_math = "\alpha + \beta = \gamma"

# Bad: Common invalid escapes
text_with_invalid = "This is \wrong"
another_invalid = "Path\to\something"
mixed = "Valid \n Invalid \k Mixed"

# Good: Using raw strings for regex and paths
windows_path_good = r"C:\Users\name\Documents\file.txt"
regex_pattern_good = r"\d+\.\d+"
word_pattern_good = r"\w+@\w+\.\w+"
latex_math_good = r"\alpha + \beta = \gamma"

# Good: Valid escape sequences
newline = "Line 1\nLine 2"
tab = "Column1\tColumn2\tColumn3"
quotes = "She said \"Hello\" to him"
backslash = "Path\\to\\file"
carriage_return = "Text\rOverwrite"

# Good: Special escape sequences
bell = "\a"
backspace = "\b"
formfeed = "\f"
vertical_tab = "\v"
null = "\0"

# Good: Unicode escapes
unicode_char = "\u0041"  # 'A'
unicode_long = "\U00000041"  # 'A'
unicode_name = "\N{LATIN CAPITAL LETTER A}"  # 'A'
hex_escape = "\x41"  # 'A'

# Good: Octal escapes
octal_a = "\101"  # 'A'
octal_null = "\0"
octal_max = "\377"  # Max value for 1 byte

# Good: Raw strings
raw_regex = r"(\d{3})-(\d{3})-(\d{4})"
raw_path = r"\\server\share\folder\file.txt"
raw_latex = r"\frac{1}{2} + \sqrt{2}"

# Good: Bytes and raw bytes
bytes_data = b"Binary\x00data"
raw_bytes = br"Raw\bytes"
raw_bytes_alt = rb"Raw\bytes"

# Good: F-strings (skipped by this rule)
name = "World"
f_string = f"Hello {name}\!"

# Good: Triple quoted strings
multiline = """
This is a multiline
string with \various escapes
"""

# Good: Escaped quotes in strings
single_in_double = "It's a nice day"
double_in_single = 'He said "Hello"'
escaped_single = 'It\'s escaped'
escaped_double = "He said \"Hello\""