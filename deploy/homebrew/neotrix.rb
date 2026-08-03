# typed: false
# frozen_string_literal: true
#
# NeoTrix — Homebrew Formula
# Usage:
#   brew tap neotrix/tap https://github.com/neotrix/neotrix
#   brew install neotrix
#
# This formula is auto-generated for the install-distribute pipeline.
# Update VERSION and checksums on each release.

class Neotrix < Formula
  desc "AI-native reasoning engine with self-evolving capability vectors"
  homepage "https://neotrix.ai"
  url "https://github.com/neotrix/neotrix/archive/refs/tags/v0.18.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"
  head "https://github.com/neotrix/neotrix.git", branch: "main"

  livecheck do
    url :stable
    strategy :github_latest
  end

  depends_on "rust" => :build

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-aarch64-apple-darwin.tar.gz"
      # sha256 must be updated per release:
      #   curl -sL https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-aarch64-apple-darwin.tar.gz | shasum -a 256
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-apple-darwin.tar.gz"
      # sha256 must be updated per release:
      #   curl -sL https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-apple-darwin.tar.gz | shasum -a 256
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    url "https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-unknown-linux-gnu.tar.gz"
    # sha256 must be updated per release:
    #   curl -sL https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-unknown-linux-gnu.tar.gz | shasum -a 256
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "neotrix"
    end
  end

  test do
    system "#{bin}/neotrix", "--help"
  end
end
