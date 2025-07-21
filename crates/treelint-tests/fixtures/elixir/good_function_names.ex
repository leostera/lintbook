defmodule Example do
  def get_user_data(id) do
    fetch_user(id)
  end
  
  def process_http_request(request) do
    handle(request)
  end
  
  def valid_function? do
    true
  end
  
  def dangerous_function! do
    :ok
  end
  
  defp private_helper do
    :helper
  end
  
  defmacro good_macro_name do
    quote do: :ok
  end
  
  defmacrop good_private_macro do
    quote do: :ok
  end
  
  # Operator functions are allowed
  def +(left, right) do
    left + right
  end
  
  def ==(left, right) do
    left == right
  end
end