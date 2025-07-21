defmodule BadCodeError do
  defexception [:message]
end

defmodule ParserError do
  defexception [:message]
end

defmodule BadHTTPResponse do
  defexception [:message]
end

defmodule HTTPHeaderException do
  defexception [:message]
end