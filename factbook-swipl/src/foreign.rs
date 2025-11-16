use crate::{Context, EngineHandle, Term};
use swipl_fli as pl;

struct MyNondetPred {
    i: i32,
}

trait Nondet {
    type Args<'a>;

    fn init(ctx: &EngineHandle) -> Self;
    fn next(&mut self, ctx: &EngineHandle, args: Self::Args<'_>) -> bool;
}

impl Nondet for MyNondetPred {
    type Args<'a> = [Term<'a>; 1];

    fn init(_: &EngineHandle) -> Self {
        Self { i: 1 }
    }

    fn next(&mut self, ctx: &EngineHandle, args: Self::Args<'_>) -> bool {
        if self.i <= 3 {
            let t = ctx.new_term().put(self.i);
            self.i += 1;
            args[0].unify_with(t)
        } else {
            false
        }
    }
}

pub extern "C" fn my_nondet_pred(t: pl::term_t, ctrl: pl::control_t) -> pl::foreign_t {
    let ctx = unsafe { EngineHandle::assume_attached() };

    match unsafe { pl::PL_foreign_control(ctrl) } as u32 {
        pl::PL_FIRST_CALL => {
            let mut state = MyNondetPred::init(&ctx);

            if state.next(&ctx, [Term::from_ptr(t)]) {
                let boxed = Box::leak(Box::new(state));
                unsafe { pl::_PL_retry_address(boxed as *mut _ as _) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_REDO => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let state = unsafe { (state_ptr as *mut MyNondetPred).as_mut() }.unwrap();

            if state.next(&ctx, [Term::from_ptr(t)]) {
                unsafe { pl::_PL_retry_address(state_ptr) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_PRUNED => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let _state = unsafe { Box::from_raw(state_ptr as _) };

            pl::TRUE as _
        },
        c => panic!("unexpected value returned from PL_foreign_control: {c}"),
    }
}
