class Lintbook < Formula
  desc "LLM-powered linter for project-local rulebooks"
  homepage "https://github.com/leostera/lintbook"
  version "0.1.0"
  license "BSD-3-Clause"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/leostera/lintbook/releases/download/v#{version}/lintbook-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 :no_check
    else
      url "https://github.com/leostera/lintbook/releases/download/v#{version}/lintbook-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 :no_check
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/leostera/lintbook/releases/download/v#{version}/lintbook-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 :no_check
    else
      odie "lintbook release archives are not published for this Linux architecture yet"
    end
  end

  def install
    bin.install "lintbook"
  end

  test do
    assert_match "lintbook", shell_output("#{bin}/lintbook --help")
  end
end
