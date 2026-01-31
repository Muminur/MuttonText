cask "muttontext" do
  version "1.0.0"
  sha256 :no_check

  url "https://github.com/muttontext/muttontext/releases/download/v#{version}/MuttonText_#{version}_universal.dmg"
  name "MuttonText"
  desc "Fast, cross-platform text expansion application"
  homepage "https://github.com/muttontext/muttontext"

  depends_on macos: ">= :monterey"

  app "MuttonText.app"

  zap trash: [
    "~/Library/Application Support/com.muttontext.app",
    "~/Library/Preferences/com.muttontext.app.plist",
    "~/Library/Caches/com.muttontext.app",
  ]
end
