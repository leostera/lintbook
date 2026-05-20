defmodule Clean do
  require Logger
  
  def good_function do
    result = calculate_something()
    Logger.info("Processing result: #{inspect(result)}")
    process_result(result)
  end
  
  def another_function do
    # This is not a debug call, just normal function
    pry(some_argument)
    continue()
  end
end