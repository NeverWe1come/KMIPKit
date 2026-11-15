from contextlib import redirect_stderr
import io
import tempfile
import unittest
from pathlib import Path

from kmipkit_validator.cli import main


ROOT = Path(__file__).resolve().parents[3]
CAPABILITIES = ROOT / "interop/shared/profiles/cosmian-kms.json"


class LiveSafetyTests(unittest.TestCase):
    def test_run_requires_a_connection_before_it_can_emit_results(self):
        with tempfile.TemporaryDirectory() as temporary:
            errors = io.StringIO()
            with redirect_stderr(errors), self.assertRaises(SystemExit):
                main(
                    [
                        "run",
                        "--capabilities",
                        str(CAPABILITIES),
                        "--output-dir",
                        temporary,
                    ]
                )
            self.assertIn("--connection", errors.getvalue())
            self.assertEqual([], list(Path(temporary).iterdir()))


if __name__ == "__main__":
    unittest.main()
