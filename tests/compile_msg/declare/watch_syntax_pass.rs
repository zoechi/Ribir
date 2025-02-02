use ribir::prelude::*;

fn main() {
  let _flow_simple = widget! {
    Flex {
      SizedBox {
        id: a,
        size: Size::zero(),
      }
      SizedBox {
        id: b,
        size: Size::zero(),
      }
    }
    finally {
      watch!(a.size)
        .subscribe(move |v| b.size = v );
    }
  };

  let _flow_handler = widget! {
    Flex {
      SizedBox {
        id: a,
        size: Size::zero(),
        on_tap: move |_| {}
      }
      SizedBox {
        id: b,
        size: a.size,
      }
      SizedBox {
        id: c,
        size: Size::zero(),
      }
    }
    finally {
      let_watch!(a.size + b.size)
        .subscribe(move |v| c.size = v);
    }

  };

  let _flow_embed = widget! {
    Flex {
      SizedBox {
        id: a,
        size: Size::zero(),
      }
      SizedBox {
        id: b,
        size: Size::zero(),
      }
      DynWidget {
        dyns: true.then(||{
          widget!{
            SizedBox {
              id: c,
              size: Size::zero(),
            }
            finally {
              watch!(a.size + b.size)
                .subscribe(move |v| c.size = v);
            }
          }
        })
      }
    }
    finally {
      watch!(a.size)
        .subscribe(move |v| b.size = v);
    }
  };

  let _fix_named_obj_moved_in_flow = widget! {
    Flex {
      SizedBox { id: a, size: Size::zero() }
      SizedBox { id: b, size: Size::zero() }
      SizedBox { id: c, size: Size::zero() }
    }
    finally {
      watch!(a.size)
        .subscribe(move |v| {
          b.size = v;
          c.size = v;
        });
    }
  };
}
