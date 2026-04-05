#!/bin/sh
set -eu

if [ "$#" -ne 4 ]; then
  echo "usage: render-homebrew-formula.sh <version> <macos-arm-sha256> <macos-intel-sha256> <linux-sha256>" >&2
  exit 1
fi

VERSION="$1"
MACOS_ARM_SHA="$2"
MACOS_INTEL_SHA="$3"
LINUX_SHA="$4"

cat <<EOF
class JiraCli < Formula
  desc "Jira Cloud CLI with keyring-backed auth and Markdown-to-ADF workflows"
  homepage "https://github.com/Rana-Faraz/jira-cli"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Rana-Faraz/jira-cli/releases/download/v${VERSION}/jira-cli-v${VERSION}-aarch64-apple-darwin.tar.gz"
      sha256 "${MACOS_ARM_SHA}"
    else
      url "https://github.com/Rana-Faraz/jira-cli/releases/download/v${VERSION}/jira-cli-v${VERSION}-x86_64-apple-darwin.tar.gz"
      sha256 "${MACOS_INTEL_SHA}"
    end
  end

  on_linux do
    url "https://github.com/Rana-Faraz/jira-cli/releases/download/v${VERSION}/jira-cli-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "${LINUX_SHA}"
  end

  def install
    bin.install "jira"
    prefix.install "README.md"
    prefix.install "LICENSE"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/jira version").strip
  end
end
EOF
