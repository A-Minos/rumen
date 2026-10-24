// Generate with tetris-fumen@1.1.3 installed outside the source tree:
// npm install --prefix target/fumen-reference --no-save --ignore-scripts tetris-fumen@1.1.3
// NODE_PATH=target/fumen-reference/node_modules node tests/generate_fixtures.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { decoder, encoder, Field, Mino } = require('tetris-fumen');
const { createActionEncoder } = require('tetris-fumen/lib/action');
const { Piece, parseRotation } = require('tetris-fumen/lib/defines');
const { Buffer: FumenBuffer } = require('tetris-fumen/lib/buffer');
const { createCommentParser } = require('tetris-fumen/lib/comments');
const version = require('tetris-fumen/package.json').version;
assert.equal(version, '1.1.3');

const cases = [];
const actionEncoder = createActionEncoder(10, 23, 1);
const facings = { reverse: 'Down', right: 'Right', spawn: 'Up', left: 'Left' };
function add(name, source) {
    const fumen = typeof source === 'string' ? source : encoder.encode(source);
    const pages = decoder.decode(fumen).map(page => {
        const field = page.field;
        let operation = null;
        if (page.operation) {
            const op = page.operation;
            const value = actionEncoder.encode({
                piece: { type: Piece[op.type], rotation: parseRotation(op.rotation), x: op.x, y: op.y },
                rise: page.flags.rise, mirror: page.flags.mirror, colorize: page.flags.colorize,
                comment: false, lock: page.flags.lock,
            });
            const index = Math.floor(value / 32) % 240;
            operation = {
                block: op.type, facing: facings[op.rotation], x: index % 10, y: Math.floor(index / 10),
                positions: Mino.from(op).positions().map(({ x, y }) => [x, 22 - y]),
            };
        }
        return {
            field: Array.from({ length: 24 }, (_, row) =>
                Array.from({ length: 10 }, (_, x) => field.at(x, 22 - row)).join('')),
            operation,
            comment: page.comment,
            flags: {
                up: page.flags.rise, mirror: page.flags.mirror,
                color_scheme: page.flags.colorize ? 'GuideLine' : 'Sega', lock: page.flags.lock,
            },
        };
    });
    cases.push({ name, fumen, pages });
}

add('documented unchanged field', 'v115@ghE8JeAgHvhAAAA');
add('documented signed deltas', 'v115@bhA8SeAAAbhAAA8ReAAAchAAA8QeAAA');
add('published seven-page example', 'v115@vhGRQYHAvItJEJmhCAUGJKJJvMJTNJGBJFKYPAUEzP?EJG98AQmqhECDdCA');
add('flag coordinate truncation regression', 'v115@ghE8JeVJJ');
add('comment length counts characters', 'v115@ghE8JeAgWDABUYCA');
add('upstream ordinary comment inheritance', 'v115@vhDAgWFAooMDEPBAAAAAPFA3XaDEEBAAAAAAAAPDAFrmAA');
add('upstream hold stock swap exhaustion', 'v115@vhGSSYXAFLDmClcJSAVDEHBEooRBMoAVBUtfBAXsBAAANrBmnBAAAAAA');
add('upstream unlocked quiz', 'v115@vhBSSYaAFLDmClcJSAVDEHBEooRBPoAVBTejWC0/AA?AAAA');

for (const type of ['I', 'L', 'O', 'Z', 'T', 'J', 'S']) {
    for (const rotation of ['spawn', 'right', 'reverse', 'left']) {
        add(`${type} ${rotation} locks into next field`, [
            { operation: { type, rotation, x: 4, y: 6 } }, {},
        ]);
    }
}

add('offboard horizontal cells follow wire array indices', 'v115@vhCVJJAAAAAA');
for (const action of ['NkH', 'VkH', 'WkH', '2kH', 'vkH', '/kH']) {
    add(`extended top field retains clear boundaries ${action}`, `v115@vhC${action}AAAAAA`);
}
add('below-field locks preserve in-field cells', [
    { operation: { type: 'I', rotation: 'right', x: 4, y: -1 } }, {}, {},
]);

const combined = Field.create('XXXX__XXXXXXXX__XXXX', 'XX_XXXXXXX');
combined.set(2, 22, 'J');
combined.set(8, 21, 'S');
add('lock clear rise mirror order', [
    { field: combined, operation: { type: 'O', rotation: 'spawn', x: 4, y: 0 }, flags: { rise: true, mirror: true } }, {},
]);
add('unlocked page ignores all events', [
    { field: combined, operation: { type: 'O', rotation: 'spawn', x: 4, y: 0 }, flags: { lock: false, rise: true, mirror: true } },
    { flags: { lock: false } },
]);
add('mirror leaves garbage untouched', [
    { field: Field.create('T________L', 'Z________S'), flags: { mirror: true } }, {},
]);
add('clear full row without active mino', [
    { field: Field.create('J_________XXXXXXXXXX', 'XXXXXXXXXX') }, {},
]);
const rising = Field.create('L_________', 'X_________');
rising.set(5, 22, 'T');
add('rise discards top and empties garbage', [{ field: rising, flags: { rise: true } }, {}]);
add('locked placement can overwrite existing cells', [
    { field: Field.create('JJJJJJJJJ_'), operation: { type: 'I', rotation: 'spawn', x: 4, y: 0 } }, {},
]);
add('unlocked preview may extend offboard', [
    { operation: { type: 'I', rotation: 'spawn', x: 0, y: 22 }, flags: { lock: false } }, {},
]);
add('repeat counter rolls over after 64 pages', Array.from({ length: 65 }, () => ({})));

let seed = 0x115c0de;
function random() {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    return seed >>> 0;
}
const palette = '_ILOZTJSX';
const varied = [];
for (let page = 0; page < 12; page++) {
    const field = Field.create();
    for (let y = -1; y < 23; y++) {
        for (let x = 0; x < 10; x++) field.set(x, y, palette[random() % palette.length]);
    }
    varied.push({ field, flags: { lock: false, colorize: page % 2 === 0 } });
}
add('all cell values and signed field changes', varied);

const comments = ['', 'a', 'ab', 'abc', 'abcd', 'abcdf', 'a \t\n', '%u4F60', '%20', 'éÿ', '你好', String.fromCodePoint(0x1f600), '\0'];
add('comment groups Unicode whitespace and inheritance', comments.flatMap(comment => [{ comment }, {}]));
add('maximum encoded comment length and following page', [{ comment: 'A'.repeat(4095) }, { comment: 'end' }]);

function quiz(name, comment, pieces, flags = {}) {
    add(name, [
        { comment, ...pieces[0] && { operation: { type: pieces[0], rotation: 'spawn', x: 4, y: 5 } }, flags },
        ...pieces.slice(1).map(type => ({ operation: { type, rotation: 'spawn', x: 4, y: 5 } })), {},
    ]);
}
quiz('quiz direct placements', '#Q=[](I)TOSZJL', ['I', 'T', 'O']);
quiz('quiz swaps held piece', '#Q=[T](I)OSZJL', ['T', 'I', 'O']);
quiz('quiz stocks current piece', '#Q=[](I)TOSZJL', ['T', 'I', 'O']);
quiz('quiz retains state on unrelated placement', '#Q=[](I)T', ['L', 'I']);
quiz('quiz does not advance when unlocked', '#Q=[](I)T', ['I', 'I'], { lock: false });
quiz('quiz formatting without placement', '#Q= [I] () TOS', [null, 'I']);
quiz('quiz empty queue and held piece', '#Q=[I]()', ['I']);
quiz('quiz continues to next sequence', '#Q=[](I);#Q=[](T)OS', ['I', 'T']);
// The public encoder rejects malformed quiz text; the decoder treats it as a comment.
const literalComments = new FumenBuffer('vhD');
for (const comment of ['#Q=not a quiz', undefined, '', undefined]) {
    literalComments.push(comment === undefined ? 0 : 92160, 3);
    if (comment !== undefined) {
        const escaped = escape(comment);
        literalComments.push(escaped.length, 2);
        for (let offset = 0; offset < escaped.length; offset += 4) {
            const chunk = escaped.slice(offset, offset + 4).padEnd(4, ' ');
            const value = [...chunk].reduce((sum, character, index) =>
                sum + createCommentParser().encode(character, index), 0);
            literalComments.push(value, 5);
        }
    }
}
add('invalid quiz remains ordinary text', `v115@${literalComments}`);

const output = {
    reference: `tetris-fumen@${version}`,
    unpaired_surrogate: encoder.encode([{ comment: '\uD800' }]),
    cases,
};
const directory = path.join(__dirname, 'fixtures');
fs.mkdirSync(directory, { recursive: true });
fs.writeFileSync(path.join(directory, 'v115.json'), JSON.stringify(output, null, 2) + '\n');
console.log(`Generated ${cases.length} cases, ${cases.reduce((sum, entry) => sum + entry.pages.length, 0)} pages`);
