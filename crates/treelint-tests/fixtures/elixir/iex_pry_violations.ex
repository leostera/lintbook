defmodule Debug do
  def problematic_function do
    result = calculate_something()
    IEx.pry()
    process_result(result)
  end
  
  def another_debug_function do
    IEx.pry
    do_something()
  end
end

defmodule WithImport do
  import IEx
  
  def function_with_pry do
    pry()
    continue_processing()
  end
end