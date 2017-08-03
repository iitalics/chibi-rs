use std;
use libc;

pub trait Sexp {
    /* checked converters */
    fn as_bool(&self) -> Option<bool>;
    fn as_fixnum(&self) -> Option<isize>;
    fn as_flonum(&self) -> Option<f32>;
    fn as_char(&self) -> Option<char>;
    fn as_string(&self) -> Option<&str>;
    fn as_bytes(&self) -> Option<&[u8]>;
    // fn as_pair(&self) -> Option<( ?, ? )>;

    /* predicate versions of above */
    fn is_bool(&self) -> bool { self.as_bool().is_some() }
    fn is_fixnum(&self) -> bool { self.as_fixnum().is_some() }
    fn is_flonum(&self) -> bool { self.as_flonum().is_some() }
    fn is_char(&self) -> bool { self.as_char().is_some() }
    fn is_string(&self) -> bool { self.as_string().is_some() }
    fn is_bytes(&self) -> bool { self.as_bytes().is_some() }
    //fn is_pair(&self) -> bool { self.as_pair().is_some() }

    /* immediates */
    fn is_true(&self) -> bool { self.as_bool().unwrap_or(false) }
    fn is_false(&self) -> bool { self.as_bool().map(|x| !x).unwrap_or(false) }
    fn is_null(&self) -> bool;
    fn is_eof(&self) -> bool;
    fn is_void(&self) -> bool;
}

mod repr {
    pub const SEXP_FALSE: usize = (0 << 8) + 62;
    pub const SEXP_TRUE: usize  = (1 << 8) + 62;
    pub const SEXP_NULL: usize  = (2 << 8) + 62;
    pub const SEXP_EOF: usize   = (3 << 8) + 62;
    pub const SEXP_VOID: usize  = (4 << 8) + 62;

    pub const SEXP_FIXNUM_MASK: usize = 1;
    pub const SEXP_IMMEDIATE_MASK: usize = 0xf;
    pub const SEXP_EXTENDED_MASK: usize = 0xff;

    pub const SEXP_ISYM_TAG: usize = 6;
    pub const SEXP_IFLO_TAG: usize = 14;
    pub const SEXP_CHAR_TAG: usize = 30;
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnrootedSexp(*mut libc::c_void);

impl UnrootedSexp {
    pub fn from_ptr(p: *mut libc::c_void) -> Self {
        UnrootedSexp(p)
    }

    fn from_imm(tag: usize) -> Self {
        UnrootedSexp(tag as *mut libc::c_void)
    }
}

impl Sexp for UnrootedSexp {
    fn as_bool(&self) -> Option<bool> {
        match self.0 as usize {
            repr::SEXP_TRUE => Some(true),
            repr::SEXP_FALSE => Some(false),
            _ => None,
        }
    }

    fn as_fixnum(&self) -> Option<isize> {
        let t = self.0 as usize;
        if t & repr::SEXP_FIXNUM_MASK == 1 {
            Some((self.0 as isize) >> 1)
        } else {
            None
        }
    }

    fn as_char(&self) -> Option<char> {
        let t = self.0 as usize;
        if t & repr::SEXP_EXTENDED_MASK == repr::SEXP_CHAR_TAG {
            Some((t >> 8) as u8 as char)
        } else {
            None
        }
    }

    fn as_flonum(&self) -> Option<f32> { None }
    fn as_string(&self) -> Option<&str> { None }
    fn as_bytes(&self) -> Option<&[u8]> { None }

    fn is_null(&self) -> bool { (self.0 as usize) == repr::SEXP_NULL }
    fn is_eof(&self) -> bool { (self.0 as usize) == repr::SEXP_EOF }
    fn is_void(&self) -> bool { (self.0 as usize) == repr::SEXP_VOID }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn imm_predicates() {
        let from_imm = UnrootedSexp::from_imm;
        fn is_0(x: &UnrootedSexp) -> bool { x.as_fixnum() == Some(0) }
        fn is_25(x: &UnrootedSexp) -> bool { x.as_fixnum() == Some(25) }
        fn is_minus2(x: &UnrootedSexp) -> bool { x.as_fixnum() == Some(-2) }
        fn is_newline(x: &UnrootedSexp) -> bool { x.as_char() == Some('\n') }
        fn is_cap_a(x: &UnrootedSexp) -> bool { x.as_char() == Some('A') }

        let objs = [
            from_imm(0x03e), // false
            from_imm(0x13e), // true
            from_imm(0x23e), // null
            from_imm(0x33e), // eof
            from_imm(0x43e), // void
            from_imm(1), // integer 0
            from_imm(51), // integer 25
            from_imm((-3i64) as usize), // integer -2
            from_imm(0xa1e), // char 0xa = '\n'
            from_imm(0x411e), // char 0x41 = 'A'
        ];

        let preds = [
            <UnrootedSexp as Sexp>::is_false,
            <UnrootedSexp as Sexp>::is_true,
            <UnrootedSexp as Sexp>::is_null,
            <UnrootedSexp as Sexp>::is_eof,
            <UnrootedSexp as Sexp>::is_void,
            is_0,
            is_25,
            is_minus2,
            is_newline,
            is_cap_a,
        ];

        for (i, &obj) in objs.iter().enumerate() {
            for (j, &pred) in preds.iter().enumerate() {
                assert!((i == j) == pred(&obj), "pred[{}] on obj[{}] = {}", j, i, (i == j));
            }
        }
    }

}
