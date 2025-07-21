defmodule Clean do
  require Logger
  
  def good_function do
    result = calculate_something()
    Logger.info("Processing result: #{inspect(result)}")
    process_result(result)
  end
  
  def function_with_kernel_inspect do
    # Kernel.inspect is used for string interpolation, not debugging
    "Value: #{inspect(some_value)}"
  end
  
  def function_with_proper_debugging do
    result = compute_result()
    Logger.debug("Computed result", result: result)
    result
  end
end