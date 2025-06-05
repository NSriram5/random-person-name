
/// An enum of character to make rust better use of pattern matching in code elsewhere. 
#[derive(Debug,Clone,Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValidChar {
    /// a
    a=0,
    /// b
    b=1,
    /// c
    c=2,
    /// d
    d=3,
    /// e
    e=4,
    /// f
    f=5,
    /// g
    g=6,
    /// h
    h=7,
    /// i
    i=8,
    /// j
    j=9,
    /// k
    k=10,
    /// l
    l=11,
    /// m
    m=12,
    /// n
    n=13,
    /// o
    o=14,
    /// p
    p=15,
    /// q
    q=16,
    /// r
    r=17,
    /// s
    s=18,
    /// t
    t=19,
    /// u
    u=20,
    /// v
    v=21,
    /// w
    w=22,
    /// x
    x=23,
    /// y
    y=24,
    /// z
    z=25,
    /// dash
    dash=26,
    /// apostrophe
    apostrophe=27,
    /// null
    null=28
}

pub const VALID_CHAR_COUNT: usize = ValidChar::VARIANTCOUNT as usize;

impl ValidChar {
    /// A helper constant to track the count of characters that are considered valid in the system.
    pub const VARIANTCOUNT: u8 = 29;
    /// A helper constant to quickly index valid characters
    pub const ALLCHARS: [ValidChar; VALID_CHAR_COUNT] = [
        ValidChar::a,
        ValidChar::b,
        ValidChar::c,
        ValidChar::d,
        ValidChar::e,
        ValidChar::f,
        ValidChar::g,
        ValidChar::h,
        ValidChar::i,
        ValidChar::j,
        ValidChar::k,
        ValidChar::l,
        ValidChar::m,
        ValidChar::n,
        ValidChar::o,
        ValidChar::p,
        ValidChar::q,
        ValidChar::r,
        ValidChar::s,
        ValidChar::t,
        ValidChar::u,
        ValidChar::v,
        ValidChar::w,
        ValidChar::x,
        ValidChar::y,
        ValidChar::z,
        ValidChar::dash,
        ValidChar::apostrophe,
        ValidChar::null
    ];
}

impl TryFrom<&char> for ValidChar {
    type Error=String;
    fn try_from(c: &char) -> Result<Self, String> {
        let input_char = c.to_lowercase().next().unwrap();
        let early_res = match input_char {
            '-' => Some(Self::dash),
            '\'' => Some(Self::apostrophe),
            '\0' => Some(Self::null),
            _ => None
        };
        if let Some(res) = early_res {return Ok(res);}
        let c_ident = c.to_lowercase().next().unwrap() as u32 - 'a' as u32;
        match c_ident {
            0 => Ok(Self::a),
            1 => Ok(Self::b),
            2 => Ok(Self::c),
            3 => Ok(Self::d),
            4 => Ok(Self::e),
            5 => Ok(Self::f),
            6 => Ok(Self::g),
            7 => Ok(Self::h),
            8 => Ok(Self::i),
            9 => Ok(Self::j),
            10 => Ok(Self::k),
            11 => Ok(Self::l),
            12 => Ok(Self::m),
            13 => Ok(Self::n),
            14 => Ok(Self::o),
            15 => Ok(Self::p),
            16 => Ok(Self::q),
            17 => Ok(Self::r),
            18 => Ok(Self::s),
            19 => Ok(Self::t),
            20 => Ok(Self::u),
            21 => Ok(Self::v),
            22 => Ok(Self::w),
            23 => Ok(Self::x),
            24 => Ok(Self::y),
            25 => Ok(Self::z),
            _ =>  Err(format!("{c} is an invalid character"))
        }
    }
}

impl From<ValidChar> for char {
    fn from(value: ValidChar) -> Self {
        match value {
            ValidChar::apostrophe => '\'',
            ValidChar::dash => '-',
            ValidChar::null => '\0',
            _ => char::from_u32(value as u32 + 'a' as u32).unwrap()
        }
    }
}

impl TryFrom<u8> for ValidChar {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..26 => Ok(
                ValidChar::try_from(&char::from_u32('a' as u32 + value as u32).unwrap()).unwrap()
            ),
            26 => Ok(ValidChar::dash),
            27 => Ok(ValidChar::apostrophe),
            28 => Ok(ValidChar::null),
            _ => Err(format!("{value} is an invalid character"))
        }
    }
}

impl From<ValidChar> for usize {
    fn from(value: ValidChar) -> Self {
        value as usize
    }
}
