defmodule Unsafe do
  def convert_user_input(input) do
    # This is dangerous - user input can create unlimited atoms
    atom = String.to_atom(input)
    process_atom(atom)
  end
  
  def convert_char_list(char_list) do
    # This is also dangerous
    atom = List.to_atom(char_list)
    process_atom(atom)
  end
end

defmodule WithImport do
  import String, only: [to_atom: 1]
  
  def dangerous_function(input) do
    # Even when imported, this is dangerous
    atom = to_atom(input)
    process_atom(atom)
  end
end