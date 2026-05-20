defmodule BadCodeError do
  defexception [:message]
end

defmodule ParserError do
  defexception [:message]
end

defmodule ValidationError do
  defexception [:message]
end