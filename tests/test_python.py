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
        codes = {symbol: value for value, symbol in enumerate("_ILOZTJSX")}
        self.assertTrue(self.corpus["cases"])
        for case in self.corpus["cases"]:
            with self.subTest(case=case["name"]):
                fumen = rumen.parse(case["fumen"])
                self.assertIsInstance(fumen, rumen.Fumen)
                self.assertEqual(fumen.version, 115)
                self.assertEqual(fumen.mode, rumen.ViewMode.Normal)
                self.assertEqual(len(fumen), len(case["pages"]))
                pages = list(fumen)
                self.assertEqual(len(pages), len(fumen))
                matrices = fumen.to_matrices()
                self.assertIsInstance(matrices, list)
                self.assertEqual(len(matrices), len(pages))
                for index, (page, expected) in enumerate(zip(pages, case["pages"])):
                    with self.subTest(page=index):
                        self.assertIsInstance(page, rumen.Page)
                        expected_matrix = [
                            [codes[cell] for cell in row] for row in expected["field"]
                        ]
                        self.assertEqual(matrices[index], expected_matrix)
                        self.assertEqual(fumen[index].to_matrix(), expected_matrix)
                        self.assertTrue(
                            all(isinstance(row, list) for row in matrices[index])
                        )
                        self.assertTrue(
                            all(
                                type(cell) is int
                                for row in matrices[index]
                                for cell in row
                            )
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

    def test_indexed_pages_and_iteration(self):
        case = next(
            case
            for case in self.corpus["cases"]
            if case["name"] == "upstream ordinary comment inheritance"
        )
        fumen = rumen.parse(case["fumen"])
        expected = [page["comment"] for page in case["pages"]]
        self.assertEqual(len(fumen), len(expected))
        for index in range(-len(fumen), len(fumen)):
            self.assertEqual(fumen[index].notes, expected[index])
        self.assertEqual([page.notes for page in fumen], expected)
        iterator = fumen.__iter__()
        self.assertIs(iter(iterator), iterator)
        self.assertEqual([page.notes for page in iterator], expected)
        self.assertEqual(
            [page.notes for page in iter(rumen.parse(case["fumen"]))], expected
        )

        class Index:
            def __index__(self):
                return 1

        self.assertEqual(fumen[Index()].notes, expected[1])
        for index in (len(fumen), -len(fumen) - 1):
            with self.subTest(index=index), self.assertRaises(IndexError):
                fumen[index]
        with self.assertRaises(TypeError):
            fumen[0.5]

        empty = rumen.parse("v115@")
        self.assertEqual(len(empty), 0)
        self.assertEqual(list(empty), [])
        self.assertEqual(empty.to_matrices(), [])
        for index in (0, -1):
            with self.subTest(index=index), self.assertRaises(IndexError):
                empty[index]

    def test_numeric_matrices_are_independent_copies(self):
        fumen = rumen.parse("v115@ghE8JeAgHvhAAAA")
        matrices = fumen.to_matrices()
        matrices[0][22][5] = 0
        matrices[0][0][0] = 8
        self.assertEqual(matrices[1][22][5], 8)
        self.assertEqual(matrices[0][1][0], 0)
        self.assertEqual(fumen[0].to_matrix()[22][5], 8)
        page = fumen[0]
        matrix = page.to_matrix()
        matrix[0][0] = 8
        self.assertEqual(page.to_matrix()[0][0], 0)

    def test_read_only_models_and_enum_keys(self):
        code = "v115@ghE8JeAgHvhAAAA"
        fumen = rumen.parse(code)
        self.assertEqual(fumen, rumen.parse(code))
        self.assertNotEqual(fumen, rumen.parse("v115@"))
        page = fumen[0]
        self.assertEqual(
            Counter(page.flag.mino for page in fumen), {rumen.Block.Empty: 2}
        )
        self.assertEqual(int(rumen.Block.Garbage), 8)
        self.assertFalse(hasattr(fumen, "pages"))
        self.assertFalse(hasattr(page, "field"))
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
        self.assertIs(rumen.parse("v115@vhAAgWAA")[0].flag.note, True)
        self.assertIs(rumen.parse("v115@vhAAgH")[0].flag.note, False)

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
