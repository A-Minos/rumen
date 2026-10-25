# rumen

**小方块，大吞吐。Rust 干重活，Python 管好用。**

用 Rust 编写的 **fumen v115 解码器**，通过 PyO3 提供 Python 绑定。输入谱面字符串或查看器链接，得到盘面、操作和注释，用来分析谱面、处理训练数据，或者接进你的方块工具。

- 每页独立的 `24 × 10` 盘面快照、方块操作与事件标志，支持注释继承和 Quiz。
- 紧凑盘面存储、原地转义、Quiz 缓冲区复用，减少重复分配与搬运。
- Python 支持按页访问、批量整数矩阵和类型提示，解码时释放 GIL。

## Python：几行就够

需要 **Python 3.9+、Rust 工具链和 uv**。以下从源码安装，在仓库根目录执行：

```sh
uv venv
uv pip install .
```

用 `.venv` 中的 Python 运行：

```python
import rumen

fumen = rumen.parse("v115@ghE8JeAgHvhAAAA")
page = fumen[0]
matrix = page.to_matrix()       # list[list[int]]，24 行 × 10 列

for page in fumen:
    print(page.notes, page.flag.positions())

matrices = fumen.to_matrices()  # list[list[list[int]]]，页 / 行 / 列
```

`fumen` 支持长度、下标、负索引和迭代。模型属性只读，导出的矩阵是独立的可变列表。

## Rust：直接接上

在消费项目的 `Cargo.toml` 中添加本地依赖，把路径指向这个仓库：

```toml
[dependencies]
rumen = { path = "../rumen" }
```

```rust
use rumen::DecodeError;

fn main() -> Result<(), DecodeError> {
    let fumen = rumen::parse("https://fumen.zui.jp/?v115@ghE8JeAgHvhAAAA")?;

    for page in &fumen.pages {
        println!("{} {:?}", page.notes, page.flag.positions());
    }
    Ok(())
}
```

`Fumen` 也实现了 `FromStr`。原生 Rust 使用不需要开启 `python` feature。

### 命令行也能用

```sh
cargo run --release -- 'v115@ghE8JeAgHvhAAAA'
```

输出完整的 Rust Debug 结构，方便检查谱面。

## 接口约定

- 盘面是当前页操作前的状态，按 `[y][x]` 访问；左上角为 `(0, 0)`，向右、向下递增，`y = 23` 为垃圾行。
- 矩阵编码 `0..8` 依次对应 `Empty, I, L, O, Z, T, J, S, Garbage`。
- `flag.location` 与 `flag.positions()` 使用同一套自上而下的坐标。`positions()` 返回方块占据的四个格子，无活动方块时为 `None`；坐标可能超出盘面。
- `page.notes` 是继承与 Quiz 推进后的可见注释，`page.flag.note` 表示该页是否显式更新了注释。
- 解码失败时，Rust 返回 `DecodeError`，Python 抛出 `rumen.DecodeError`（`ValueError` 子类）。
- 目前仅提供 v115 解码，不含编码器。

## 开发

```sh
cargo fmt --check
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings

# 先按上文安装 Python 包；以下为 Linux / macOS 路径
.venv/bin/python -B -m unittest discover -s tests -p test_python.py
```

协议行为参考 [tetris-fumen](https://github.com/knewjade/tetris-fumen)，相关许可见 [LICENSES/tetris-fumen.txt](LICENSES/tetris-fumen.txt)。
