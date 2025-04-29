pagrs-core: core mechanics for pagrs (page rotation)
=====================================================

This crate defines the main traits as well as provides the rotation and controlling mechanism for pagrs.
It does not define any pages itself, those are inside crates of their own.

main elements
--------------

- trait `Page`: required trait for any kind of page that is usable by pagrs
- struct `PageRotator`: main working horse, managing which page to show at any given time. There can currently be only one active
- struct `PageController`: an object to send control commands to the page rotator, e.g. moving to the next page or previous page


getting started
-----------------

- to set up the `PageRotator` the following is required:
  - the owned display object (currently limited to a `Ssd1306Async`!) as factory-method parameter
  - the maximum amount of pages type parameter
  - `DI`, `SIZE`, `MODE` type parameters, implicitly coming from the display object
- run the async method `init()` on the `PageRotator`
- afterward the individual pages need to be registered
- when all is configured run the async method `.rotate()` (which requires the async spawner) on the `PageRotator`

  **info**: the `rotate` method never ends but starts its own endless loop for controlling the page rotation

```rust
fn main() {
    /* let page = [...] */
    /* let display = [...] */
    let mut pagr = PageRotator::<5, _, _, _>::new(display);
    pagr.init().await.unwrap();
    
    let _ = pagr.add_page(&mut page);
    pagr.rotate(spawner).await
}
```

**info**: It is important, that the page objects are defined _before_ the `PageRotator` object, as dropping is done in the 
inverse order (last defined object first) and the pages must live longer then the `PageRotator`


write your own page
---------------------

Custom pages can easily be created by implementing the `Page` trait.

### implement rendering

three methods come with a default implementation, but obviously can be overwritten. 
The most important method, to actually draw something on the display, must be implemented: `render`.

The method receives a `DrawTarget` from the `embedded-graphics` crate as parameter and can make use of all the methods
defined by that trait to define the output. The `embedded-graphics` crate comes with several 'native' element that can be leveraged for
creating the desired design. See e.g. `pagrs-bmp` or `pagrs-text`. But also quite low-level drawing is possible, see `pagrs-matrix`.

### state handling

The call to `render` only provides the display to render onto, it does not bear state. The page must keep track of its
internal state.
If that state is influenced by something outside the page itself appropriate access protection and guards must be used 
to ensure the code is kept free from race conditions.


### framerate

one of the methods of the trait is `frames_per_second`. It defaults to returning a constant of 24 frames per second.
the individual page can change that framerate. The lower the frame rate the less resources it takes. Especially static 
content can render with a low framerate.

The decision about the framerate belongs to the page, as some internal state calculation might depend on it.


### lifecycle of a page

- each page has to be created before registering to the pagrs controller and need to stay alive for the whole duration of the application.
- everytime a page is rotated in the `activated()` method is called.
- as long as the page is active the `render()` method is called for each frame.
- everytime a page is rotated out the `deactivated()` method is called.

