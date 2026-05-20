defmodule Safe do
  def convert_safely(input) do
    # These are safe alternatives
    case String.to_existing_atom(input) do
      {:ok, atom} -> process_atom(atom)
      {:error, _} -> :invalid_atom
    end
  end
  
  def use_string_directly(input) do
    # Using strings directly is also safe
    process_string(input)
  end
  
  def use_predefined_atom do
    # Hardcoded atoms are fine
    :my_predefined_atom
  end
  
  def check_existing_atom(input) do
    # This is safe - only converts existing atoms
    List.to_existing_atom(input)
  end
end