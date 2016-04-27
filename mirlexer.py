from pygments.lexer import RegexLexer
from pygments.token import Whitespace, Comment, Name, Keyword, Number, \
    String, Punctuation, Operator, Text


class RustMirLexer(RegexLexer):
    name = 'Rust MIR'
    filenames = []
    aliases = ['rustmir']
    mimetypes = ['text/x-rust-mir']

    tokens = {
        'root': [
            (r'\s+', Whitespace),
            (r'//(.*?)\n', Comment.Single),
            (r'/\*', Comment.Multiline, 'comment'),
            (r"bb\d+:", Name.Function),
            (r'let\b', Keyword.Declaration),
            (r'(fn|as|const|goto|mut|resume|return|unwind|asm!)\b', Keyword),
            (r'(Box|Len|Add|Sub|Mul|Div|Rem|BitXor|BitAnd|BitOr|Shl|Shr|Eq|'
             r'Lt|Le|Ne|Ge|Gt|Not|Neg|drop|switch|switchInt)\b', Name.Builtin),
            (r'(true|false)\b', Keyword.Constant),
            (r'(u8|u16|u32|u64|i8|i16|i32|i64|usize|isize|f32|f64|str|bool)\b',
             Keyword.Type),
            (r"""'(\\['"\\nrt]|\\x[0-7][0-9a-fA-F]|\\0"""
             r"""|\\u\{[0-9a-fA-F]{1,6}\}|.)'""", String.Char),
            (r"""b'(\\['"\\nrt]|\\x[0-9a-fA-F]{2}|\\0"""
             r"""|\\u\{[0-9a-fA-F]{1,6}\}|.)'""", String.Char),
            (r'[0-9][0-9_]*(\.[0-9_]+[eE][+\-]?[0-9_]+|'
             r'\.[0-9_]*(?!\.)|[eE][+\-]?[0-9_]+)', Number.Float, 'number_lit'),
            (r'[0-9][0-9_]*', Number.Integer, 'number_lit'),
            (r'b"', String, 'bytestring'),
            (r'"', String, 'string'),
            (r"'(static|<.*?>|[a-zA-Z_]\w*)", Name.Attribute),
            (r'[{}()\[\],.;]', Punctuation),
            (r'[+\-*/%&|<>^!~@=:?]', Operator),
            (r'[a-zA-Z_]\w*', Name),
        ],
        'comment': [
            (r'[^*/]+', Comment.Multiline),
            (r'/\*', Comment.Multiline, '#push'),
            (r'\*/', Comment.Multiline, '#pop'),
            (r'[*/]', Comment.Multiline),
        ],
        'number_lit': [
            (r'[ui](8|16|32|64|size)', Number, '#pop'),
            (r'f(32|64)', Number, '#pop'),
            (r'', Text, '#pop'),
        ],
        'string': [
            (r'"', String, '#pop'),
            (r"\\['\"\\nrt]|\\x[0-7][0-9a-fA-F]|\\0|\\u\{[0-9a-fA-F]{1,6}\}",
             String.Escape),
            (r'[^\\"]+', String),
            (r'\\', String),
        ],
        'bytestring': [
            (r'"', String, '#pop'),
            (r"\\['\"\\nrt]|\\x[0-7][0-9a-fA-F]|\\0", String.Escape),
            (r'[^\\"]+', String),
            (r'\\', String),
        ],
    }
