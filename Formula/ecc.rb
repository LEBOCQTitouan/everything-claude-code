class Ecc < Formula
  desc "CLI to set up and manage Claude Code configuration"
  homepage "https://github.com/LEBOCQTitouan/everything-claude-code"
  license "MIT"

  head "https://github.com/LEBOCQTitouan/everything-claude-code.git", branch: "main"

  depends_on "node" => :optional

  def install
    # Copy everything to libexec so install.sh can resolve its own source dirs
    libexec.install Dir["*"]

    # Create ecc wrapper in bin
    (bin/"ecc").write <<~SH
      #!/usr/bin/env bash
      exec "#{libexec}/install.sh" "$@"
    SH
  end

  test do
    output = shell_output("#{bin}/ecc 2>&1", 1)
    assert_match "install", output
    assert_match "init", output
  end
end
