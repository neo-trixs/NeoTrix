cask "neotrix-desktop" do
  version "0.18.0"

  desc "AI-native developer toolkit desktop app (Tauri)"
  homepage "https://neotrix.ai"

  # TODO(release-checklist 3.5): CDN/artifact hosting not yet live — replace
  # the placeholder URL + sha256 at publish time. Filled by CI (see
  # .github/workflows/release.yml, release-checklist 1.6).
  #
  #   ditto --cachedir /dev/null "/path/to/NeoTrix.app" /dev/null
  #   shasum -a 256 "/path/to/NeoTrix.dmg"
  if Hardware::CPU.arm?
    url "https://releases.neotrix.ai/desktop/#{version}/NeoTrix-arm64.dmg"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  else
    url "https://releases.neotrix.ai/desktop/#{version}/NeoTrix-x64.dmg"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  name "NeoTrix"
  name "NeoTrix Desktop"
  app "NeoTrix.app"
  auto_updates true
end
