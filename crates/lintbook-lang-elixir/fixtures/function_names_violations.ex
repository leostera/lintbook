defmodule Example do
  def getUserData(id) do
    # This should be get_user_data
    fetch_user(id)
  end
  
  def processHTTPRequest(request) do
    # This should be process_http_request
    handle(request)
  end
  
  def badFunction do
    :not_ok
  end
  
  def another_Good_Function do
    :also_not_ok
  end
  
  defmacro badMacroName do
    quote do: :ok
  end
  
  defp privateHelperFunction do
    :helper
  end
end