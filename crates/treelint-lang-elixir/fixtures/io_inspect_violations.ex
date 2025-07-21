defmodule Debug do
  def problematic_function do
    result = calculate_something()
    IO.inspect(result, label: "Debug result")
    process_result(result)
  end
  
  def debug_with_options do
    data = %{a: 1, b: 2}
    IO.inspect(data, label: "Debug", pretty: true, limit: :infinity)
    process_data(data)
  end
end

defmodule WithImport do
  import IO
  
  def problematic_function do
    result = calculate_something()
    inspect(result)
    process_result(result)
  end
end