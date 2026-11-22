const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');
const { test } = require('node:test');

const html = readFileSync(join(__dirname, '..', 'viewer', 'studio.html'), 'utf8');
const script = html.match(/<script>\n([\s\S]*?)<\/script>/)[1];

function fakeElement() {
    return {
        children: [], style: {}, innerHTML: '', innerText: '', value: '',
        classList: { add() {}, remove() {}, toggle() {} },
        appendChild(child) { this.children.push(child); },
        remove() {},
    };
}

function loadStudio() {
    const elements = new Map();
    const document = {
        getElementById: id => elements.get(id) ?? elements.set(id, fakeElement()).get(id),
        createElement: fakeElement,
        querySelectorAll: () => [],
    };
    const context = vm.createContext({
        document, localStorage: { getItem: () => null }, window: {},
        THREE: { MeshStandardMaterial: class {}, MeshNormalMaterial: class {} },
    });
    vm.runInContext(script, context);
    return { context, elements, run: code => vm.runInContext(code, context) };
}

const HOSTILE = '"><img src=x onerror=alert(1)>';
const ENTITIES = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'" };
const tagShape = markup => markup.match(/<[^>]*>/g).map(tag => tag.replace(/"[^"]*"/g, '""'));
const visibleText = markup => markup.replace(/<[^>]*>/g, '').replace(/&(?:#(\d+)|#x([\da-f]+)|([a-z]+));/gi,
    (_, dec, hex, name) => dec ? String.fromCharCode(dec) : hex ? String.fromCharCode(parseInt(hex, 16)) : ENTITIES[name]);

function renderModelList(name) {
    const studio = loadStudio();
    studio.context.models = [{ name: name + '.glb', folder: name, category: name, size: 4, url: '/models/x.glb' }];
    studio.run('allModels = models; renderAccordionList();');
    const [group] = studio.elements.get('model-list').children;
    return group.children[0].innerHTML + group.children[1].children[0].innerHTML;
}

function renderTextureGrid(name) {
    const studio = loadStudio();
    studio.context.textures = Array.from({ length: 81 },
        () => ({ name: name + '.png', format: 'PNG Image', size: 4, preview: '/textures/x.png' }));
    studio.run('allTextures = textures; renderTexturesGrid();');
    const grid = studio.elements.get('textures-grid');
    grid.children.at(-1).onclick();
    const cards = grid.children.filter(child => child.className === 'tex-card');
    assert.equal(cards.length, 81);
    return cards[0].innerHTML + cards[80].innerHTML;
}

test('hostile model and texture file names render as text, not as elements or attributes', () => {
    for (const render of [renderModelList, renderTextureGrid]) {
        const hostile = render(HOSTILE);
        assert.deepEqual(tagShape(hostile), tagShape(render('plain name')));
        assert.ok(visibleText(hostile).includes(HOSTILE), `${render.name} must still show the name`);
    }
});
