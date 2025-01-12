let wasm_bindgen;
(function() {
    const __exports = {};
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }
    let wasm = undefined;

    const heap = new Array(128).fill(undefined);

    heap.push(undefined, null, true, false);

    function getObject(idx) { return heap[idx]; }

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

    if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

    let cachedUint8ArrayMemory0 = null;

    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_export_0(addHeapObject(e));
        }
    }

    let WASM_VECTOR_LEN = 0;

    const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

    const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
        ? function (arg, view) {
        return cachedTextEncoder.encodeInto(arg, view);
    }
        : function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    });

    function passStringToWasm0(arg, malloc, realloc) {

        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }

        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = encodeString(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    let cachedDataViewMemory0 = null;

    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function dropObject(idx) {
        if (idx < 132) return;
        heap[idx] = heap_next;
        heap_next = idx;
    }

    function takeObject(idx) {
        const ret = getObject(idx);
        dropObject(idx);
        return ret;
    }

    let cachedFloat32ArrayMemory0 = null;

    function getFloat32ArrayMemory0() {
        if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
            cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
        }
        return cachedFloat32ArrayMemory0;
    }

    function getArrayF32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    let cachedInt32ArrayMemory0 = null;

    function getInt32ArrayMemory0() {
        if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
            cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
        }
        return cachedInt32ArrayMemory0;
    }

    function getArrayI32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    let cachedUint32ArrayMemory0 = null;

    function getUint32ArrayMemory0() {
        if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
            cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
        }
        return cachedUint32ArrayMemory0;
    }

    function getArrayU32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => {
        wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b)
    });

    function makeMutClosure(arg0, arg1, dtor, f) {
        const state = { a: arg0, b: arg1, cnt: 1, dtor };
        const real = (...args) => {
            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbindgen_export_4.get(state.dtor)(a, state.b);
                    CLOSURE_DTORS.unregister(state);
                } else {
                    state.a = a;
                }
            }
        };
        real.original = state;
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches && builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }
    function __wbg_adapter_58(arg0, arg1) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbindgen_export_5(retptr, arg0, arg1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }

    function __wbg_adapter_61(arg0, arg1, arg2) {
        wasm.__wbindgen_export_6(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_66(arg0, arg1, arg2) {
        wasm.__wbindgen_export_7(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_822(arg0, arg1, arg2, arg3) {
        wasm.__wbindgen_export_8(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
    }

    const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

    const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];

    const TrapFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_trap_free(ptr >>> 0, 1));
    /**
     * A struct representing a Trap
     */
    class Trap {

        static __wrap(ptr) {
            ptr = ptr >>> 0;
            const obj = Object.create(Trap.prototype);
            obj.__wbg_ptr = ptr;
            TrapFinalization.register(obj, obj.__wbg_ptr, obj);
            return obj;
        }

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            TrapFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_trap_free(ptr, 0);
        }
        /**
         * A marker method to indicate that an object is an instance of the `Trap`
         * class.
         */
        static __wbg_wasmer_trap() {
            wasm.trap___wbg_wasmer_trap();
        }
    }
    __exports.Trap = Trap;

    const WebHandleFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_webhandle_free(ptr >>> 0, 1));
    /**
     * Our handle to the web app from JavaScript.
     */
    class WebHandle {

        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            WebHandleFinalization.unregister(this);
            return ptr;
        }

        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_webhandle_free(ptr, 0);
        }
        /**
         * Installs a panic hook, then returns.
         */
        constructor() {
            const ret = wasm.webhandle_new();
            this.__wbg_ptr = ret >>> 0;
            WebHandleFinalization.register(this, this.__wbg_ptr, this);
            return this;
        }
        /**
         * Call this once from JavaScript to start your app.
         * @param {HTMLCanvasElement} canvas
         * @returns {Promise<void>}
         */
        start(canvas) {
            const ret = wasm.webhandle_start(this.__wbg_ptr, addHeapObject(canvas));
            return takeObject(ret);
        }
        destroy() {
            wasm.webhandle_destroy(this.__wbg_ptr);
        }
        /**
         * The JavaScript can check whether or not your app has crashed:
         * @returns {boolean}
         */
        has_panicked() {
            const ret = wasm.webhandle_has_panicked(this.__wbg_ptr);
            return ret !== 0;
        }
        /**
         * @returns {string | undefined}
         */
        panic_message() {
            try {
                const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
                wasm.webhandle_panic_message(retptr, this.__wbg_ptr);
                var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
                var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
                let v1;
                if (r0 !== 0) {
                    v1 = getStringFromWasm0(r0, r1).slice();
                    wasm.__wbindgen_export_3(r0, r1 * 1, 1);
                }
                return v1;
            } finally {
                wasm.__wbindgen_add_to_stack_pointer(16);
            }
        }
        /**
         * @returns {string | undefined}
         */
        panic_callstack() {
            try {
                const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
                wasm.webhandle_panic_callstack(retptr, this.__wbg_ptr);
                var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
                var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
                let v1;
                if (r0 !== 0) {
                    v1 = getStringFromWasm0(r0, r1).slice();
                    wasm.__wbindgen_export_3(r0, r1 * 1, 1);
                }
                return v1;
            } finally {
                wasm.__wbindgen_add_to_stack_pointer(16);
            }
        }
    }
    __exports.WebHandle = WebHandle;

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);

                } catch (e) {
                    if (module.headers.get('Content-Type') != 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else {
                        throw e;
                    }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);

        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };

            } else {
                return instance;
            }
        }
    }

    function __wbg_get_imports() {
        const imports = {};
        imports.wbg = {};
        imports.wbg.__wbg_BigInt_83b60d3772ed53fe = function(arg0) {
            const ret = BigInt(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_activeElement_ea31ecc5423c6046 = function(arg0) {
            const ret = getObject(arg0).activeElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_activeTexture_446c979476d36a40 = function(arg0, arg1) {
            getObject(arg0).activeTexture(arg1 >>> 0);
        };
        imports.wbg.__wbg_activeTexture_aec8c249ceb838d2 = function(arg0, arg1) {
            getObject(arg0).activeTexture(arg1 >>> 0);
        };
        imports.wbg.__wbg_addEventListener_b9481c2c2cab6047 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_altKey_d5409f5ddaa29593 = function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        };
        imports.wbg.__wbg_altKey_d54599b3b6b6cf22 = function(arg0) {
            const ret = getObject(arg0).altKey;
            return ret;
        };
        imports.wbg.__wbg_appendChild_d22bc7af6b96b3f1 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_apply_0595e14e01b58931 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).apply(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_apply_e185aee3133b64b8 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.apply(getObject(arg0), getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_arrayBuffer_1be504e3eb62daa4 = function(arg0) {
            const ret = getObject(arg0).arrayBuffer();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_arrayBuffer_d0ca2ad8bda0039b = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).arrayBuffer();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_at_479807bfddde3a33 = function(arg0, arg1) {
            const ret = getObject(arg0).at(arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_attachShader_4dc5977795b5d865 = function(arg0, arg1, arg2) {
            getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
        };
        imports.wbg.__wbg_attachShader_9b79a4896fee779d = function(arg0, arg1, arg2) {
            getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
        };
        imports.wbg.__wbg_bindBuffer_e9412cc77f8130d6 = function(arg0, arg1, arg2) {
            getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindBuffer_ff7c55f1062014bc = function(arg0, arg1, arg2) {
            getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindFramebuffer_c89f5adcd05acda2 = function(arg0, arg1, arg2) {
            getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindFramebuffer_fbd7ce3580c64aab = function(arg0, arg1, arg2) {
            getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindTexture_8b97cf7511a725d0 = function(arg0, arg1, arg2) {
            getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindTexture_f65d2e377e3de352 = function(arg0, arg1, arg2) {
            getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
        };
        imports.wbg.__wbg_bindVertexArrayOES_19ed43bbe1241f7a = function(arg0, arg1) {
            getObject(arg0).bindVertexArrayOES(getObject(arg1));
        };
        imports.wbg.__wbg_bindVertexArray_67a807a1cd64976a = function(arg0, arg1) {
            getObject(arg0).bindVertexArray(getObject(arg1));
        };
        imports.wbg.__wbg_bind_e443b204773403b4 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).bind(getObject(arg1), getObject(arg2), getObject(arg3));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_blendEquationSeparate_26681d98390d0057 = function(arg0, arg1, arg2) {
            getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        };
        imports.wbg.__wbg_blendEquationSeparate_e81d45aebb0a6f22 = function(arg0, arg1, arg2) {
            getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        };
        imports.wbg.__wbg_blendFuncSeparate_0031130a17fd5eb8 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        };
        imports.wbg.__wbg_blendFuncSeparate_4d5cc402dcf7389f = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        };
        imports.wbg.__wbg_blockSize_6464e214800294a9 = function(arg0) {
            const ret = getObject(arg0).blockSize;
            return ret;
        };
        imports.wbg.__wbg_blur_51f415004ecbe327 = function() { return handleError(function (arg0) {
            getObject(arg0).blur();
        }, arguments) };
        imports.wbg.__wbg_body_8d7d8c4aa91dcad8 = function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_bottom_72e7516e4f4e156a = function(arg0) {
            const ret = getObject(arg0).bottom;
            return ret;
        };
        imports.wbg.__wbg_bufferData_0643498950a2292f = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
        };
        imports.wbg.__wbg_bufferData_7e2b6059c35c9291 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
        };
        imports.wbg.__wbg_buffer_61b7ce01341d7f88 = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_button_12b22015f2d5993d = function(arg0) {
            const ret = getObject(arg0).button;
            return ret;
        };
        imports.wbg.__wbg_byteLength_f9964250eb4a86b5 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_call_500db948e69c7330 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_call_b0d8e36992d9900d = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_cancelAnimationFrame_5f7904867f6ab804 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).cancelAnimationFrame(arg1);
        }, arguments) };
        imports.wbg.__wbg_cancel_32c379e69f397185 = function(arg0) {
            getObject(arg0).cancel();
        };
        imports.wbg.__wbg_changedTouches_86448a1d3a872098 = function(arg0) {
            const ret = getObject(arg0).changedTouches;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_clearColor_7e5806f100e4cd7a = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_clearColor_d58166c97d5eef07 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_clearDepth_09770ed9850b53ca = function(arg0, arg1) {
            getObject(arg0).clearDepth(arg1);
        };
        imports.wbg.__wbg_clearDepth_2634fb35e857706d = function(arg0, arg1) {
            getObject(arg0).clearDepth(arg1);
        };
        imports.wbg.__wbg_clearInterval_998151d924a72ab2 = function(arg0, arg1) {
            getObject(arg0).clearInterval(arg1);
        };
        imports.wbg.__wbg_clear_16ffdcc1a1d6f0c9 = function(arg0, arg1) {
            getObject(arg0).clear(arg1 >>> 0);
        };
        imports.wbg.__wbg_clear_c182acb53176ea8b = function(arg0, arg1) {
            getObject(arg0).clear(arg1 >>> 0);
        };
        imports.wbg.__wbg_clientX_18c5fbacc6398ad8 = function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        };
        imports.wbg.__wbg_clientX_f73b86b8aba3591d = function(arg0) {
            const ret = getObject(arg0).clientX;
            return ret;
        };
        imports.wbg.__wbg_clientY_0974153484cf0d09 = function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        };
        imports.wbg.__wbg_clientY_af033356579f2b9c = function(arg0) {
            const ret = getObject(arg0).clientY;
            return ret;
        };
        imports.wbg.__wbg_clipboardData_d188aa1b339f8637 = function(arg0) {
            const ret = getObject(arg0).clipboardData;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_clipboard_b634c08b310bca2e = function(arg0) {
            const ret = getObject(arg0).clipboard;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_colorMask_04bc7392c9d1b568 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        };
        imports.wbg.__wbg_colorMask_401f99e62155b996 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        };
        imports.wbg.__wbg_compileShader_afcc43901f14a922 = function(arg0, arg1) {
            getObject(arg0).compileShader(getObject(arg1));
        };
        imports.wbg.__wbg_compileShader_fab2df50ae89c5e1 = function(arg0, arg1) {
            getObject(arg0).compileShader(getObject(arg1));
        };
        imports.wbg.__wbg_constructor_dbc1fa8bf53335ed = function(arg0) {
            const ret = getObject(arg0).constructor;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_contentBoxSize_c8dcd6b272f821ba = function(arg0) {
            const ret = getObject(arg0).contentBoxSize;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_contentRect_6fadfee6731ac5df = function(arg0) {
            const ret = getObject(arg0).contentRect;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createBuffer_567b536a03db30d2 = function(arg0) {
            const ret = getObject(arg0).createBuffer();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createBuffer_8692729b8ac9caaf = function(arg0) {
            const ret = getObject(arg0).createBuffer();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createElement_89923fcb809656b7 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createFramebuffer_346cd4e0b98b15c4 = function(arg0) {
            const ret = getObject(arg0).createFramebuffer();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createFramebuffer_a3361909a5a3c966 = function(arg0) {
            const ret = getObject(arg0).createFramebuffer();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createProgram_b8f69529220fb50b = function(arg0) {
            const ret = getObject(arg0).createProgram();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createProgram_e2141127012594b0 = function(arg0) {
            const ret = getObject(arg0).createProgram();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createShader_442f69b8f536a786 = function(arg0, arg1) {
            const ret = getObject(arg0).createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createShader_809bd3abe629ad7a = function(arg0, arg1) {
            const ret = getObject(arg0).createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createTexture_3c9e731e954515fa = function(arg0) {
            const ret = getObject(arg0).createTexture();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createTexture_677a150f3f985ce0 = function(arg0) {
            const ret = getObject(arg0).createTexture();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createVertexArrayOES_950dd712f273bb06 = function(arg0) {
            const ret = getObject(arg0).createVertexArrayOES();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createVertexArray_68ae270682fc14aa = function(arg0) {
            const ret = getObject(arg0).createVertexArray();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_crypto_ed58b8e10a292839 = function(arg0) {
            const ret = getObject(arg0).crypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_ctrlKey_5a324c8556fbce1c = function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        };
        imports.wbg.__wbg_ctrlKey_5c308955b0d5492d = function(arg0) {
            const ret = getObject(arg0).ctrlKey;
            return ret;
        };
        imports.wbg.__wbg_cullFace_d68398a8ce2f6fe3 = function(arg0, arg1) {
            getObject(arg0).cullFace(arg1 >>> 0);
        };
        imports.wbg.__wbg_cullFace_dbad3db56721e436 = function(arg0, arg1) {
            getObject(arg0).cullFace(arg1 >>> 0);
        };
        imports.wbg.__wbg_dataTransfer_c29d7d69c9576def = function(arg0) {
            const ret = getObject(arg0).dataTransfer;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_data_8980cafa6731c6b5 = function(arg0, arg1) {
            const ret = getObject(arg1).data;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_deleteBuffer_783d60e842697847 = function(arg0, arg1) {
            getObject(arg0).deleteBuffer(getObject(arg1));
        };
        imports.wbg.__wbg_deleteBuffer_bf5a34580654a42a = function(arg0, arg1) {
            getObject(arg0).deleteBuffer(getObject(arg1));
        };
        imports.wbg.__wbg_deleteFramebuffer_73b3cd0be3d68d24 = function(arg0, arg1) {
            getObject(arg0).deleteFramebuffer(getObject(arg1));
        };
        imports.wbg.__wbg_deleteFramebuffer_f91ceb4755e7bed3 = function(arg0, arg1) {
            getObject(arg0).deleteFramebuffer(getObject(arg1));
        };
        imports.wbg.__wbg_deleteProgram_3ca13ed49ca24a48 = function(arg0, arg1) {
            getObject(arg0).deleteProgram(getObject(arg1));
        };
        imports.wbg.__wbg_deleteProgram_47e8c8c7f0923d3d = function(arg0, arg1) {
            getObject(arg0).deleteProgram(getObject(arg1));
        };
        imports.wbg.__wbg_deleteShader_e1f71043508b6951 = function(arg0, arg1) {
            getObject(arg0).deleteShader(getObject(arg1));
        };
        imports.wbg.__wbg_deleteShader_e4fe2574d9c3afab = function(arg0, arg1) {
            getObject(arg0).deleteShader(getObject(arg1));
        };
        imports.wbg.__wbg_deleteTexture_36653aa53d4a29e9 = function(arg0, arg1) {
            getObject(arg0).deleteTexture(getObject(arg1));
        };
        imports.wbg.__wbg_deleteTexture_eaf729f97b59aaf4 = function(arg0, arg1) {
            getObject(arg0).deleteTexture(getObject(arg1));
        };
        imports.wbg.__wbg_deltaMode_b2e9bb0dca5cf196 = function(arg0) {
            const ret = getObject(arg0).deltaMode;
            return ret;
        };
        imports.wbg.__wbg_deltaX_5c26d3b55d406732 = function(arg0) {
            const ret = getObject(arg0).deltaX;
            return ret;
        };
        imports.wbg.__wbg_deltaY_1683a859ce933add = function(arg0) {
            const ret = getObject(arg0).deltaY;
            return ret;
        };
        imports.wbg.__wbg_depthFunc_351a7bf1596d4061 = function(arg0, arg1) {
            getObject(arg0).depthFunc(arg1 >>> 0);
        };
        imports.wbg.__wbg_depthFunc_70ac0cb861c8a03b = function(arg0, arg1) {
            getObject(arg0).depthFunc(arg1 >>> 0);
        };
        imports.wbg.__wbg_depthMask_0ff63f0d4501072b = function(arg0, arg1) {
            getObject(arg0).depthMask(arg1 !== 0);
        };
        imports.wbg.__wbg_depthMask_3daac0e40564953e = function(arg0, arg1) {
            getObject(arg0).depthMask(arg1 !== 0);
        };
        imports.wbg.__wbg_detachShader_17ec388351e24b61 = function(arg0, arg1, arg2) {
            getObject(arg0).detachShader(getObject(arg1), getObject(arg2));
        };
        imports.wbg.__wbg_detachShader_444b29fa4e93147e = function(arg0, arg1, arg2) {
            getObject(arg0).detachShader(getObject(arg1), getObject(arg2));
        };
        imports.wbg.__wbg_devicePixelContentBoxSize_f91b326c21f7e3d5 = function(arg0) {
            const ret = getObject(arg0).devicePixelContentBoxSize;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_devicePixelRatio_973abafaa5e8254b = function(arg0) {
            const ret = getObject(arg0).devicePixelRatio;
            return ret;
        };
        imports.wbg.__wbg_disableVertexAttribArray_1bf5b473f133c8ab = function(arg0, arg1) {
            getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
        };
        imports.wbg.__wbg_disableVertexAttribArray_f49780d5b42e6b0d = function(arg0, arg1) {
            getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
        };
        imports.wbg.__wbg_disable_2f09f593bf0f5573 = function(arg0, arg1) {
            getObject(arg0).disable(arg1 >>> 0);
        };
        imports.wbg.__wbg_disable_302597eacd491d44 = function(arg0, arg1) {
            getObject(arg0).disable(arg1 >>> 0);
        };
        imports.wbg.__wbg_disconnect_6e7f07912b7a73c6 = function(arg0) {
            getObject(arg0).disconnect();
        };
        imports.wbg.__wbg_document_f11bc4f7c03e1745 = function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_done_f22c1561fa919baa = function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        };
        imports.wbg.__wbg_drawArrays_01e26acf05821932 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
        };
        imports.wbg.__wbg_drawArrays_32d97bfaf282c738 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
        };
        imports.wbg.__wbg_drawBuffersWEBGL_f5eea4913dbb3ac0 = function(arg0, arg1) {
            getObject(arg0).drawBuffersWEBGL(getObject(arg1));
        };
        imports.wbg.__wbg_drawBuffers_5a72890eb9a6161d = function(arg0, arg1) {
            getObject(arg0).drawBuffers(getObject(arg1));
        };
        imports.wbg.__wbg_drawElements_28e4f5037fe8c665 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).drawElements(arg1 >>> 0, arg2, arg3 >>> 0, arg4);
        };
        imports.wbg.__wbg_drawElements_3d61ffb17b84bc9d = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).drawElements(arg1 >>> 0, arg2, arg3 >>> 0, arg4);
        };
        imports.wbg.__wbg_elementFromPoint_4d710989dedbb0f3 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).elementFromPoint(arg1, arg2);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_enableVertexAttribArray_211547224fc25327 = function(arg0, arg1) {
            getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
        };
        imports.wbg.__wbg_enableVertexAttribArray_60827f2a43782639 = function(arg0, arg1) {
            getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
        };
        imports.wbg.__wbg_enable_2bacfac56e802b11 = function(arg0, arg1) {
            getObject(arg0).enable(arg1 >>> 0);
        };
        imports.wbg.__wbg_enable_a7767e03f7973ca8 = function(arg0, arg1) {
            getObject(arg0).enable(arg1 >>> 0);
        };
        imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_error_a252c810ff5d4f45 = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_error_fab41a42d22bf2bc = function(arg0) {
            console.error(getObject(arg0));
        };
        imports.wbg.__wbg_eval_cd0c386c3899dd07 = function() { return handleError(function (arg0, arg1) {
            const ret = eval(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_exports_47e00f52aac40ed0 = function(arg0) {
            const ret = getObject(arg0).exports;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_exports_fa88afc6fdcf5a43 = function(arg0) {
            const ret = WebAssembly.Module.exports(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_fetch_e26fdd92ea39f634 = function(arg0, arg1) {
            const ret = getObject(arg0).fetch(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_files_95d9491da88a54b5 = function(arg0) {
            const ret = getObject(arg0).files;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_focus_35fe945f7268dd62 = function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments) };
        imports.wbg.__wbg_force_82b5a32305812290 = function(arg0) {
            const ret = getObject(arg0).force;
            return ret;
        };
        imports.wbg.__wbg_framebufferRenderbuffer_a51d38203e558ea9 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
        };
        imports.wbg.__wbg_framebufferRenderbuffer_d198a03c2c5c7581 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
        };
        imports.wbg.__wbg_framebufferTexture2D_86a2063326486ec7 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
        };
        imports.wbg.__wbg_framebufferTexture2D_a1a6486dde56610f = function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
        };
        imports.wbg.__wbg_framebufferTextureLayer_12c11d57e6a4c64f = function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5);
        };
        imports.wbg.__wbg_generateMipmap_82e271fcb6f70fdc = function(arg0, arg1) {
            getObject(arg0).generateMipmap(arg1 >>> 0);
        };
        imports.wbg.__wbg_generateMipmap_b78fb575f19cb920 = function(arg0, arg1) {
            getObject(arg0).generateMipmap(arg1 >>> 0);
        };
        imports.wbg.__wbg_getActiveAttrib_ea3aced2192efa64 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getActiveAttrib(getObject(arg1), arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getActiveAttrib_fd7eaa2661d7dcc0 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getActiveAttrib(getObject(arg1), arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getActiveUniform_03ea1aca5aa73a89 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getActiveUniform(getObject(arg1), arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getActiveUniform_c2b1f2275b45927e = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getActiveUniform(getObject(arg1), arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getAttribLocation_a3fffeb134874426 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).getAttribLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        };
        imports.wbg.__wbg_getAttribLocation_e104a96119fd0bbd = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).getAttribLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        };
        imports.wbg.__wbg_getBoundingClientRect_05c4b9e3701bb372 = function(arg0) {
            const ret = getObject(arg0).getBoundingClientRect();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getComputedStyle_8e58bbd76370e2b1 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getComputedStyle(getObject(arg1));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getContext_5eaf5645cd6acb46 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getData_a7e2258d0a97ce17 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getData(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getError_440987efeb6b80e1 = function(arg0) {
            const ret = getObject(arg0).getError();
            return ret;
        };
        imports.wbg.__wbg_getError_74cd1cb3c131ece0 = function(arg0) {
            const ret = getObject(arg0).getError();
            return ret;
        };
        imports.wbg.__wbg_getExtension_b96fdd5b9c1f7271 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getExtension(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getExtension_f31653ddc3f1cef9 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getExtension(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getItem_badd23d1a06e7b19 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getParameter_6f7bc820485dbae4 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getParameter(arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getParameter_fc177c1d22da930b = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).getParameter(arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getProgramInfoLog_70d114345e15d2c1 = function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getProgramInfoLog_760af7d6753bc699 = function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getProgramParameter_8a6b724d42d813b3 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getProgramParameter_d328869400b82698 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getPropertyValue_66c16bac362c6d90 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_getPrototypeOf_331ddadec9037c44 = function() { return handleError(function (arg0) {
            const ret = Reflect.getPrototypeOf(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getRandomValues_bcb4912f16000dc4 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).getRandomValues(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_getShaderInfoLog_23dd787b504d5f4e = function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getShaderInfoLog_da62e75d61fbf8a8 = function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_getShaderParameter_e9098a633e6cf618 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getShaderParameter_f9c66f7ac8114c69 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getSupportedExtensions_3ce4548166177471 = function(arg0) {
            const ret = getObject(arg0).getSupportedExtensions();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getSupportedExtensions_6f069a552bc69ef2 = function(arg0) {
            const ret = getObject(arg0).getSupportedExtensions();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getTime_ab8b72009983c537 = function(arg0) {
            const ret = getObject(arg0).getTime();
            return ret;
        };
        imports.wbg.__wbg_getTimezoneOffset_ec375e661c590c7a = function(arg0) {
            const ret = getObject(arg0).getTimezoneOffset();
            return ret;
        };
        imports.wbg.__wbg_getUniformLocation_95f3933486db473c = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getUniformLocation_b9be4fbca76ab9a4 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_get_014bbb75e258e645 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_get_198bfcfce5b3a38f = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_get_5933bdeb54f47bcc = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).get(arg1 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_9aa3dff3f0266054 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_get_bbccf8970793c087 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_dfac72a5ffb577cc = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_getindex_26733aea78acb768 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return ret;
        };
        imports.wbg.__wbg_hash_4227a319264c4ca1 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_headers_24e3e19fe3f187c0 = function(arg0) {
            const ret = getObject(arg0).headers;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_headers_786276f5fbbdb28a = function(arg0) {
            const ret = getObject(arg0).headers;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_height_08fd44318e18021d = function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        };
        imports.wbg.__wbg_height_f36c36e27347cf38 = function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        };
        imports.wbg.__wbg_hidden_2f28ae7f1a034fbc = function(arg0) {
            const ret = getObject(arg0).hidden;
            return ret;
        };
        imports.wbg.__wbg_host_7131cd3aac9f8fd5 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).host;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_hostname_b4908a01df1be9f2 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hostname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_href_d04a5d3c2e6b361a = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_id_87f2e8c82a04a251 = function(arg0, arg1) {
            const ret = getObject(arg1).id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_identifier_14d5888db18610bb = function(arg0) {
            const ret = getObject(arg0).identifier;
            return ret;
        };
        imports.wbg.__wbg_imports_c3f52125aa13f5ce = function(arg0) {
            const ret = WebAssembly.Module.imports(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_inlineSize_60da5bea0a6275d2 = function(arg0) {
            const ret = getObject(arg0).inlineSize;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Element_0f1680908791f190 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Error_2b29c5b4afac4e22 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Function_6c3d8e3611ccdf21 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Function;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Global_e313ccaff9f24325 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebAssembly.Global;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_HtmlInputElement_47b3e827f364773c = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Memory_26dac6caf8b9bbf1 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebAssembly.Memory;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Object_0d0cec232ff037c4 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_ResizeObserverEntry_145c533d0e7cf2f4 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ResizeObserverEntry;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_ResizeObserverSize_8c7b496ccb6db38e = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ResizeObserverSize;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Response_d3453657e10c4300 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Table_46eb8a45ca3a827b = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebAssembly.Table;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_TypeError_cbba6ac46ad5e2bb = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof TypeError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_WebGl2RenderingContext_ed03a40cd6d9a6c5 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebGL2RenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_WebGlRenderingContext_934db43ae44dbdac = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebGLRenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Window_d2514c6a7ee7ba60 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_isComposing_34930e03980aa623 = function(arg0) {
            const ret = getObject(arg0).isComposing;
            return ret;
        };
        imports.wbg.__wbg_isComposing_e6c0e75812df1d39 = function(arg0) {
            const ret = getObject(arg0).isComposing;
            return ret;
        };
        imports.wbg.__wbg_is_e442492d1fb7967b = function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_item_7c8b01a4bd9230d3 = function(arg0, arg1) {
            const ret = getObject(arg0).item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_items_9e6108f345463d2c = function(arg0) {
            const ret = getObject(arg0).items;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_iterator_23604bb983791576 = function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_keyCode_e673401ed53dfc2c = function(arg0) {
            const ret = getObject(arg0).keyCode;
            return ret;
        };
        imports.wbg.__wbg_key_9a40d4f6defa675b = function(arg0, arg1) {
            const ret = getObject(arg1).key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_lastModified_0094dc85b1157ef5 = function(arg0) {
            const ret = getObject(arg0).lastModified;
            return ret;
        };
        imports.wbg.__wbg_left_d79d7167a89a5169 = function(arg0) {
            const ret = getObject(arg0).left;
            return ret;
        };
        imports.wbg.__wbg_length_65d1cd11729ced11 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_86e2f100fef1fecc = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_a2ea1b46bcec2b5e = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_c4528fc455e58194 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_d65cf0786bfc5739 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_linkProgram_9b1029885a37b70d = function(arg0, arg1) {
            getObject(arg0).linkProgram(getObject(arg1));
        };
        imports.wbg.__wbg_linkProgram_bcf6286423b25b5c = function(arg0, arg1) {
            getObject(arg0).linkProgram(getObject(arg1));
        };
        imports.wbg.__wbg_localStorage_9ca2da984fd56239 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).localStorage;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_location_b2ec7e36fec8a8ff = function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_log_0cc1b7768397bcfe = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_log_cb9e190acc5753fb = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbg_mark_7438147ce31e9d4b = function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbg_matchMedia_4adca948756a5784 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_matches_a69a36077c4f07ad = function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        };
        imports.wbg.__wbg_matches_f7ab6204c1d7b30d = function(arg0) {
            const ret = getObject(arg0).matches;
            return ret;
        };
        imports.wbg.__wbg_measure_fb7825c11612c823 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_export_3(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_export_3(deferred1_0, deferred1_1, 1);
            }
        }, arguments) };
        imports.wbg.__wbg_message_7bde112094278773 = function(arg0) {
            const ret = getObject(arg0).message;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_metaKey_90fbd812345a7e0c = function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        };
        imports.wbg.__wbg_metaKey_de1f08a4d1e84bd1 = function(arg0) {
            const ret = getObject(arg0).metaKey;
            return ret;
        };
        imports.wbg.__wbg_msCrypto_0a36e2ec3a343d26 = function(arg0) {
            const ret = getObject(arg0).msCrypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_name_37e12d7b980bc5bd = function(arg0, arg1) {
            const ret = getObject(arg1).name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_name_b3419a921956d8ab = function(arg0, arg1) {
            const ret = getObject(arg1).name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_navigator_0fe968937104eaa7 = function(arg0) {
            const ret = getObject(arg0).navigator;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new0_55477545727914d9 = function() {
            const ret = new Date();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_0565c3001775c60a = function() { return handleError(function (arg0) {
            const ret = new ResizeObserver(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_254fa9eac11932ae = function() {
            const ret = new Array();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_3d446df9155128ef = function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wbg_adapter_822(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = state0.b = 0;
            }
        };
        imports.wbg.__wbg_new_3ff5b33b1ce712df = function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_41257536af60ed14 = function(arg0) {
            const ret = new Date(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_688846f374351c92 = function() {
            const ret = new Object();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_803172d71ba4127d = function() { return handleError(function (arg0) {
            const ret = new WebAssembly.Module(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_8335b3b33fdfee42 = function() { return handleError(function (arg0, arg1) {
            const ret = new WebAssembly.Instance(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
            const ret = new Error();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_c8fc2c35bff0c270 = function() {
            const ret = new Error();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newnoargs_fd9e4bf8be2bc16d = function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_4b01f207bed23fc0 = function(arg0, arg1, arg2) {
            const ret = new Int8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_5910bdf845a168eb = function(arg0, arg1, arg2) {
            const ret = new Uint32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_6991ab0478cc4a43 = function(arg0, arg1, arg2) {
            const ret = new Int32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_69ec77b20853ae02 = function(arg0, arg1, arg2) {
            const ret = new Uint16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_b0192e1adfca2df1 = function(arg0, arg1, arg2) {
            const ret = new Int16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_ba35896968751d91 = function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_f113a96374814bb2 = function(arg0, arg1, arg2) {
            const ret = new Float32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithlength_34ce8f1051e74449 = function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithlength_759c7b9d6a7a314f = function(arg0) {
            const ret = new Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithstrandinit_a1f6583f20e4faff = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_newwithtext_f2ebd2622e9d451b = function() { return handleError(function (arg0, arg1) {
            const ret = new SpeechSynthesisUtterance(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_next_01dd9234a5bf6d05 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_next_137428deb98342b0 = function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_node_02999533c4ea02e3 = function(arg0) {
            const ret = getObject(arg0).node;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_now_2c95c9de01293173 = function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        };
        imports.wbg.__wbg_now_62a101fe35b60230 = function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        };
        imports.wbg.__wbg_observe_71a44d88a2880088 = function(arg0, arg1, arg2) {
            getObject(arg0).observe(getObject(arg1), getObject(arg2));
        };
        imports.wbg.__wbg_of_924412d32367b13d = function(arg0) {
            const ret = Array.of(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_offsetTop_eb7c481e74c6fa50 = function(arg0) {
            const ret = getObject(arg0).offsetTop;
            return ret;
        };
        imports.wbg.__wbg_ok_4cacdb33ce54895f = function(arg0) {
            const ret = getObject(arg0).ok;
            return ret;
        };
        imports.wbg.__wbg_open_94e1e54493f5c069 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_origin_8c23d49bc1f609e9 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_performance_2e69ce813a883f21 = function(arg0) {
            const ret = getObject(arg0).performance;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_performance_7a3ffd0b17f663ad = function(arg0) {
            const ret = getObject(arg0).performance;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_pixelStorei_7c93ee0ad7bf0763 = function(arg0, arg1, arg2) {
            getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
        };
        imports.wbg.__wbg_pixelStorei_7dc331e4d85de1a3 = function(arg0, arg1, arg2) {
            getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
        };
        imports.wbg.__wbg_port_14b0bf6b72b06565 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).port;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_preventDefault_3c86e59772d015e6 = function(arg0) {
            getObject(arg0).preventDefault();
        };
        imports.wbg.__wbg_process_5c1d670bc53614b8 = function(arg0) {
            const ret = getObject(arg0).process;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_protocol_ceaedd334dc7dbaf = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).protocol;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_push_6edad0df4b546b2c = function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_queueMicrotask_2181040e064c0dc8 = function(arg0) {
            queueMicrotask(getObject(arg0));
        };
        imports.wbg.__wbg_queueMicrotask_ef9ac43769cbcc4f = function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_randomFillSync_ab2cfe79ebbf2740 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).randomFillSync(takeObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_a9ca9f05245321f0 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments) };
        imports.wbg.__wbg_remove_530b4f3163f72a83 = function(arg0) {
            getObject(arg0).remove();
        };
        imports.wbg.__wbg_requestAnimationFrame_169cbbda5861d9ca = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_require_79b1e9274cde3c87 = function() { return handleError(function () {
            const ret = module.require;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_resolve_0bf7c44d641804f9 = function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_right_74bde7fc03836700 = function(arg0) {
            const ret = getObject(arg0).right;
            return ret;
        };
        imports.wbg.__wbg_scissor_63c22bd552b53b16 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).scissor(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_scissor_eebb3b755c95ca32 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).scissor(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_search_feca3869d55ecd5c = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_setAttribute_148e0e65e20e5f27 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setItem_37bd15ddee3594af = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setProperty_0eb9705cf1b05650 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_set_1d80752d0d5f0b21 = function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        };
        imports.wbg.__wbg_set_23d69db4e5c66a6e = function(arg0, arg1, arg2) {
            getObject(arg0).set(getObject(arg1), arg2 >>> 0);
        };
        imports.wbg.__wbg_set_4e647025551483bd = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_set_aa8f7a765a0a2e5f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_setautofocus_8b7469c890d65742 = function(arg0, arg1) {
            getObject(arg0).autofocus = arg1 !== 0;
        };
        imports.wbg.__wbg_setbody_64920df008e48adc = function(arg0, arg1) {
            getObject(arg0).body = getObject(arg1);
        };
        imports.wbg.__wbg_setbox_2c55cd020a2888a8 = function(arg0, arg1) {
            getObject(arg0).box = __wbindgen_enum_ResizeObserverBoxOptions[arg1];
        };
        imports.wbg.__wbg_setheight_16d76e7fa9d506ea = function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        };
        imports.wbg.__wbg_setmethod_cfc7f688ba46a6be = function(arg0, arg1, arg2) {
            getObject(arg0).method = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setmode_cd03637eb7da01e0 = function(arg0, arg1) {
            getObject(arg0).mode = __wbindgen_enum_RequestMode[arg1];
        };
        imports.wbg.__wbg_setpitch_575e6c4d3c08396d = function(arg0, arg1) {
            getObject(arg0).pitch = arg1;
        };
        imports.wbg.__wbg_setrate_b2b113bd358677fc = function(arg0, arg1) {
            getObject(arg0).rate = arg1;
        };
        imports.wbg.__wbg_settabIndex_52ec33536f016c41 = function(arg0, arg1) {
            getObject(arg0).tabIndex = arg1;
        };
        imports.wbg.__wbg_settype_e19ab551722d5681 = function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setvalue_3b1dde3cd4256a74 = function(arg0, arg1) {
            getObject(arg0).value = getObject(arg1);
        };
        imports.wbg.__wbg_setvalue_44c59c360ad57cf0 = function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setvolume_aff584ffa2c97910 = function(arg0, arg1) {
            getObject(arg0).volume = arg1;
        };
        imports.wbg.__wbg_setwidth_c588fe07a7982aca = function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        };
        imports.wbg.__wbg_shaderSource_3bbf44221529c149 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_shaderSource_6a657afd48edb05a = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_shiftKey_0d6625838238aee8 = function(arg0) {
            const ret = getObject(arg0).shiftKey;
            return ret;
        };
        imports.wbg.__wbg_shiftKey_4b30f68655b97001 = function(arg0) {
            const ret = getObject(arg0).shiftKey;
            return ret;
        };
        imports.wbg.__wbg_size_5ead5cc358246113 = function(arg0) {
            const ret = getObject(arg0).size;
            return ret;
        };
        imports.wbg.__wbg_size_da39fd5e748fa8df = function(arg0) {
            const ret = getObject(arg0).size;
            return ret;
        };
        imports.wbg.__wbg_speak_d88fa5f225a44c0a = function(arg0, arg1) {
            getObject(arg0).speak(getObject(arg1));
        };
        imports.wbg.__wbg_speechSynthesis_26d3bf90f6a508c7 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).speechSynthesis;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_stack_f328dd5815220aeb = function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_0be7472e492ad3e3 = function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_THIS_1a6eb482d12c9bfb = function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_SELF_1dc398a895c82351 = function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_WINDOW_ae1c80c7eea8d64a = function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_statusText_613aac5c001080c1 = function(arg0, arg1) {
            const ret = getObject(arg1).statusText;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_status_317f53bc4c7638df = function(arg0) {
            const ret = getObject(arg0).status;
            return ret;
        };
        imports.wbg.__wbg_stopPropagation_da43a41fec77962c = function(arg0) {
            getObject(arg0).stopPropagation();
        };
        imports.wbg.__wbg_style_53bb2d762dd1c030 = function(arg0) {
            const ret = getObject(arg0).style;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_subarray_46adeb9b86949d12 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_texImage2D_488bd0838560f2fd = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
        }, arguments) };
        imports.wbg.__wbg_texImage2D_d83566263a20c144 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            getObject(arg0).texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
        }, arguments) };
        imports.wbg.__wbg_texParameteri_45603287be57d25e = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        };
        imports.wbg.__wbg_texParameteri_d550886a76f21258 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        };
        imports.wbg.__wbg_texStorage2D_cde5cb7abf420f39 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        };
        imports.wbg.__wbg_texStorage3D_fc26ec90a9bea55a = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
        };
        imports.wbg.__wbg_texSubImage2D_0eeb9856a37cc769 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments) };
        imports.wbg.__wbg_texSubImage2D_355ed8d7c2b07c22 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
        }, arguments) };
        imports.wbg.__wbg_texSubImage2D_7af37da149ecfb8e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
        }, arguments) };
        imports.wbg.__wbg_texSubImage3D_bbfb28b10d74c5fb = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments) };
        imports.wbg.__wbg_texSubImage3D_d877836539c045af = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
        }, arguments) };
        imports.wbg.__wbg_then_0438fad860fe38e1 = function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_0ffafeddf0e182a4 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_toString_a491ccf7be1ca5c9 = function(arg0) {
            const ret = getObject(arg0).toString();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_toString_e52cae46e417625f = function(arg0, arg1, arg2) {
            const ret = getObject(arg1).toString(arg2);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_top_640e0509d882f0ee = function(arg0) {
            const ret = getObject(arg0).top;
            return ret;
        };
        imports.wbg.__wbg_touches_464d67ccc79e7632 = function(arg0) {
            const ret = getObject(arg0).touches;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_trap_new = function(arg0) {
            const ret = Trap.__wrap(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_type_1cad8117b64accc6 = function(arg0, arg1) {
            const ret = getObject(arg1).type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_type_394bc3bf9b919d18 = function(arg0, arg1) {
            const ret = getObject(arg1).type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_type_3fb8eddcdbcd42c9 = function(arg0) {
            const ret = getObject(arg0).type;
            return ret;
        };
        imports.wbg.__wbg_uniform1fv_235e94c6d70ecdf1 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform1fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform1fv_8416cc54f32944bf = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform1fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform1i_33a6ced29e8c7297 = function(arg0, arg1, arg2) {
            getObject(arg0).uniform1i(getObject(arg1), arg2);
        };
        imports.wbg.__wbg_uniform1i_fd66f39a37e6a753 = function(arg0, arg1, arg2) {
            getObject(arg0).uniform1i(getObject(arg1), arg2);
        };
        imports.wbg.__wbg_uniform1iv_5ac1f0357bf549e0 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform1iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform1iv_7bf17a21f0740ca2 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform1iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform1uiv_a84239dd03ea1d32 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform1uiv(getObject(arg1), getArrayU32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform2f_78492045e002e4eb = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform2f(getObject(arg1), arg2, arg3);
        };
        imports.wbg.__wbg_uniform2f_8174f619e86c3ba2 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform2f(getObject(arg1), arg2, arg3);
        };
        imports.wbg.__wbg_uniform2fv_4656b178cf5fa4c1 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform2fv_f9d9552b515ed3b7 = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform3fv_2e8d4216dfb82f6d = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform3fv_bab07e5c9f85179f = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform4fv_be28454f5049854f = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniform4fv_f7d3df09330a128e = function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
        };
        imports.wbg.__wbg_uniformMatrix2fv_17cab4f6d3030e4a = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_uniformMatrix2fv_89b5eee6863a8da9 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_uniformMatrix3fv_072dfda2d6a0e388 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_uniformMatrix3fv_c4d861a040ef1853 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_uniformMatrix4fv_b684a40949b2ff0b = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_uniformMatrix4fv_fa5c91b7cee9bfd5 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        };
        imports.wbg.__wbg_url_5327bc0a41a9b085 = function(arg0, arg1) {
            const ret = getObject(arg1).url;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_useProgram_1a5a4be134db012a = function(arg0, arg1) {
            getObject(arg0).useProgram(getObject(arg1));
        };
        imports.wbg.__wbg_useProgram_88e7787408765ccf = function(arg0, arg1) {
            getObject(arg0).useProgram(getObject(arg1));
        };
        imports.wbg.__wbg_userAgent_918b064b4cd32842 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_validate_5bb9e0985a63c4c6 = function() { return handleError(function (arg0) {
            const ret = WebAssembly.validate(getObject(arg0));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_value_47fde8ea2d9fdcd5 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbg_value_4c32fd138a88eee2 = function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_value_ae662e50f0c8e11b = function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_versions_c71aa1626a93e0a1 = function(arg0) {
            const ret = getObject(arg0).versions;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_vertexAttribDivisorANGLE_2712e437f242895b = function(arg0, arg1, arg2) {
            getObject(arg0).vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
        };
        imports.wbg.__wbg_vertexAttribDivisor_615c5c0ab239e1af = function(arg0, arg1, arg2) {
            getObject(arg0).vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
        };
        imports.wbg.__wbg_vertexAttribPointer_1f280ac2d8994592 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        };
        imports.wbg.__wbg_vertexAttribPointer_c6b1ccfa43bbca96 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        };
        imports.wbg.__wbg_viewport_1ca83fff581a8f22 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).viewport(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_viewport_770469a07e2d9772 = function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).viewport(arg1, arg2, arg3, arg4);
        };
        imports.wbg.__wbg_width_0d7b0b5ad3c2009f = function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        };
        imports.wbg.__wbg_width_9927e6a7adb23d6d = function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        };
        imports.wbg.__wbg_writeText_e65e98e75a2a92b8 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).writeText(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_from_u128 = function(arg0, arg1) {
            const ret = BigInt.asUintN(64, arg0) << BigInt(64) | BigInt.asUintN(64, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbindgen_boolean_get = function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
            return ret;
        };
        imports.wbg.__wbindgen_cb_drop = function(arg0) {
            const obj = takeObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        };
        imports.wbg.__wbindgen_closure_wrapper12118 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 3882, __wbg_adapter_66);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper2342 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 998, __wbg_adapter_58);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper2344 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 998, __wbg_adapter_61);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper2346 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 998, __wbg_adapter_61);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_function_table = function() {
            const ret = wasm.__wbindgen_export_4;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_in = function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_is_bigint = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        };
        imports.wbg.__wbindgen_is_function = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        };
        imports.wbg.__wbindgen_is_object = function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        };
        imports.wbg.__wbindgen_is_string = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        };
        imports.wbg.__wbindgen_is_undefined = function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        };
        imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_lt = function(arg0, arg1) {
            const ret = getObject(arg0) < getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_memory = function() {
            const ret = wasm.memory;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_neg = function(arg0) {
            const ret = -getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbindgen_number_new = function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
            takeObject(arg0);
        };
        imports.wbg.__wbindgen_rethrow = function(arg0) {
            throw takeObject(arg0);
        };
        imports.wbg.__wbindgen_shr = function(arg0, arg1) {
            const ret = getObject(arg0) >> getObject(arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_throw = function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        };

        return imports;
    }

    function __wbg_init_memory(imports, memory) {

    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedDataViewMemory0 = null;
        cachedFloat32ArrayMemory0 = null;
        cachedInt32ArrayMemory0 = null;
        cachedUint32ArrayMemory0 = null;
        cachedUint8ArrayMemory0 = null;



        return wasm;
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (typeof module !== 'undefined') {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();

        __wbg_init_memory(imports);

        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }

        const instance = new WebAssembly.Instance(module, imports);

        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;


        if (typeof module_or_path !== 'undefined') {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (typeof module_or_path === 'undefined' && typeof script_src !== 'undefined') {
            module_or_path = script_src.replace(/\.js$/, '_bg.wasm');
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        __wbg_init_memory(imports);

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    wasm_bindgen = Object.assign(__wbg_init, { initSync }, __exports);

})();
