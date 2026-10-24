"""Run against an installed wheel: python -B -m unittest discover -s tests -p test_python.py."""

import json
import unittest
from collections import Counter
from importlib.resources import files
from pathlib import Path

import rumen


class PythonBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture = Path(__file__).with_name("fixtures") / "v115.json"
        cls.corpus = json.loads(fixture.read_text(encoding="utf-8"))

    def test_reference_corpus_through_python_objects(self):
        symbols = {
            rumen.Block.Empty: "_",
            rumen.Block.I: "I",
            rumen.Block.L: "L",
            rumen.Block.O: "O",
            rumen.Block.Z: "Z",
            rumen.Block.T: "T",
            rumen.Block.J: "J",
            rumen.Block.S: "S",
            rumen.Block.Garbage: "X",
        }
        self.assertTrue(self.corpus["cases"])
        for case in self.corpus["cases"]:
            with self.subTest(case=case["name"]):
                fumen = rumen.parse(case["fumen"])
                self.assertIsInstance(fumen, rumen.Fumen)
                self.assertEqual(fumen.version, 115)
                self.assertEqual(fumen.mode, rumen.ViewMode.Normal)
                pages = fumen.pages
                self.assertIsInstance(pages, list)
                self.assertEqual(len(pages), len(case["pages"]))
                for index, (page, expected) in enumerate(zip(pages, case["pages"])):
                    with self.subTest(page=index):
                        self.assertIsInstance(page, rumen.Page)
                        field = page.field
                        self.assertIsInstance(field, list)
                        self.assertTrue(all(isinstance(row, list) for row in field))
                        self.assertEqual(
                            ["".join(symbols[cell] for cell in row) for row in field],
                            expected["field"],
                        )
                        self.assertEqual(page.notes, expected["comment"])
                        flag = page.flag
                        self.assertIsInstance(flag, rumen.Flag)
                        for name in ("up", "mirror", "lock"):
                            self.assertIs(getattr(flag, name), expected["flags"][name])
                        self.assertEqual(
                            flag.color_scheme,
                            getattr(
                                rumen.ColorScheme, expected["flags"]["color_scheme"]
                            ),
                        )
                        operation = expected["operation"]
                        if operation is None:
                            self.assertEqual(flag.mino, rumen.Block.Empty)
                            self.assertIsNone(flag.face)
                            self.assertIsNone(flag.location)
                            self.assertIsNone(flag.positions())
                        else:
                            self.assertEqual(
                                flag.mino, getattr(rumen.Block, operation["block"])
                            )
                            self.assertEqual(
                                flag.face, getattr(rumen.Facing, operation["facing"])
                            )
                            location = flag.location
                            self.assertIsInstance(location, rumen.Location)
                            self.assertEqual(
                                (location.x, location.y),
                                (operation["x"], operation["y"]),
                            )
                            positions = flag.positions()
                            self.assertIsInstance(positions, list)
                            self.assertEqual(
                                sorted(positions),
                                sorted(map(tuple, operation["positions"])),
                            )

    def test_snapshot_ownership_equality_and_enum_keys(self):
        code = "v115@ghE8JeAgHvhAAAA"
        fumen = rumen.parse(code)
        self.assertEqual(fumen, rumen.parse(code))
        self.assertNotEqual(fumen, rumen.parse("v115@"))
        page = fumen.pages[0]
        self.assertEqual(
            Counter(page.field[22]), {rumen.Block.Empty: 5, rumen.Block.Garbage: 5}
        )
        pages = fumen.pages
        pages.clear()
        self.assertEqual(len(fumen.pages), 2)
        field = page.field
        field[22][5] = rumen.Block.Empty
        self.assertEqual(page.field[22][5], rumen.Block.Garbage)
        for instance, name, value in [
            (fumen, "version", 110),
            (page, "notes", "changed"),
            (page.flag, "lock", False),
        ]:
            with self.subTest(attribute=name), self.assertRaises(AttributeError):
                setattr(instance, name, value)

    def test_view_modes_and_explicit_comment_flag(self):
        for prefix, mode in [
            ("v", rumen.ViewMode.Normal),
            ("m", rumen.ViewMode.Mini),
            ("d", rumen.ViewMode.List),
            ("D", rumen.ViewMode.MiniList),
        ]:
            with self.subTest(prefix=prefix):
                decoded = rumen.parse(
                    input=f"https://fumen.zui.jp/?{prefix}115@ghE8?JeAgH&unused=1"
                )
                self.assertEqual(decoded.mode, mode)
        self.assertIs(rumen.parse("v115@vhAAgWAA").pages[0].flag.note, True)
        self.assertIs(rumen.parse("v115@vhAAgH").pages[0].flag.note, False)

    def test_decoder_failures_are_value_errors_with_original_messages(self):
        cases = [
            ("v110@vhAAAA", "unsupported fumen version 110"),
            ("v115@A", "truncated fumen"),
            ("v115@AA", "invalid block -8"),
            (self.corpus["unpaired_surrogate"], "unpaired UTF-16 surrogate"),
        ]
        for code, message in cases:
            with (
                self.subTest(code=code),
                self.assertRaises(rumen.DecodeError) as caught,
            ):
                rumen.parse(code)
            self.assertIsInstance(caught.exception, ValueError)
            self.assertIn(message, str(caught.exception))
        for value in (None, 115, b"v115@"):
            with self.subTest(value=value), self.assertRaises(TypeError):
                rumen.parse(value)

    def test_wheel_includes_typing_information(self):
        package = files("rumen")
        self.assertTrue(package.joinpath("py.typed").is_file())
        self.assertTrue(package.joinpath("__init__.pyi").is_file())
        self.assertEqual(rumen.Fumen.__module__, "rumen")
        self.assertEqual(rumen.DecodeError.__module__, "rumen")


if __name__ == "__main__":
    unittest.main()
