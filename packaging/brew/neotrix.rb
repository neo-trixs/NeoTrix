# typed: false
# frozen_string_literal: true
#
# NeoTrix — Homebrew Formula (CLI)
# Usage:
#   brew tap neotrix/neotrix https://github.com/neotrix/neotrix
#   brew install neotrix
#
# Template for the release pipeline. Update VERSION and the per-arch sha256
# placeholders on each release (release-checklist 3.5 / 1.6).
# A live copy also lives at deploy/homebrew/neotrix.rb — keep in sync.

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
      # TODO(release-checklist 3.5): fill real checksum at publish time.
      #   curl -sL https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-aarch64-apple-darwin.tar.gz | shasum -a 256
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-apple-darwin.tar.gz"
      # TODO(release-checklist 3.5): fill real checksum at publish time.
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    url "https://github.com/neotrix/neotrix/releases/download/v0.18.0/neotrix-x86_64-unknown-linux-gnu.tar.gz"
    # TODO(release-checklist 3.5): fill real checksum at publish time.
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
