#[derive(Debug, toasty::Model)]
struct User {
    #[key]
    #[auto]
    id: u64,

    name: String,

    #[unique]
    email: String,

    #[has_many]
    posts: toasty::HasMany<Post>,
}

#[derive(Debug, toasty::Model)]
struct Post {
    #[key]
    #[auto]
    id: u64,

    title: String,

    #[index]
    user_id: u64,
    #[belongs_to(key = user_id, references = id)]
    user: toasty::BelongsTo<User>,

    #[index]
    event_id: u64,
    #[belongs_to(key = event_id, references = id)]
    event: toasty::BelongsTo<Event>,

    #[index]
    product_id: u64,
    #[belongs_to(key = product_id, references = id)]
    product: toasty::BelongsTo<Product>,
}

#[derive(Debug, toasty::Model)]
struct Event {
    #[key]
    #[auto]
    id: u64,

    name: String,
}

#[derive(Debug, toasty::Model)]
struct Product {
    #[key]
    #[auto]
    id: u64,

    name: String,
}

#[tokio::main]
async fn main() -> toasty::Result<()> {
    // Build a Db handle, registering all models in this crate
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite::memory:")
        .await?;

    // Create tables based on registered models
    db.push_schema().await?;

    // Create an event and product
    let event = toasty::create!(Event {
        name: "Conference 2024",
    })
    .exec(&mut db)
    .await?;

    let product = toasty::create!(Product {
        name: "Widget",
    })
    .exec(&mut db)
    .await?;

    // Create a user
    let user = toasty::create!(User {
        name: "Alice",
        email: "alice@example.com",
    })
    .exec(&mut db)
    .await?;

    // Create a post associated with user, event, and product
    toasty::create!(Post {
        title: "My First Post",
        user: &user,
        event: &event,
        product: &product,
    })
    .exec(&mut db)
    .await?;

    println!("Created user: {:?}", user.name);

    let loaded_user = User::filter_by_id(user.id)
        .first()
        .exec(&mut db)
        .await?;

    println!("Found user: {:?}", loaded_user);
    // this prints: Some(User { id: 1, name: "Alice", email: "alice@example.com", posts: <not loaded> })

    let loaded_user_include_posts = User::filter_by_id(user.id)
        .include(User::fields().posts())
        .first()
        .exec(&mut db)
        .await?;
    println!("Found user: {:?}", loaded_user_include_posts);
    // This prints: Found user: Some(User { id: 1, name: "Alice", email: "alice@example.com", posts: [Post { id: 1, title: "My First Post", user_id: 1, user: <not loaded>, event_id: 1, event: <not loaded>, product_id: 1, product: <not loaded> }] })
    // And this is fine, here we could iterate on a slice of the posts
    for p in loaded_user_include_posts.unwrap().posts.get() {
        println!("Got post {:?}", p);
        // This prints: Got post Post { id: 1, title: "My First Post", user_id: 1, user: <not loaded>, event_id: 1, event: <not loaded>, product_id: 1, product: <not loaded> }
    }

    let loaded_user_include_posts_with_event = User::filter_by_id(user.id)
        .include(User::fields().posts())
        .include(User::fields().posts().event())
        .first()
        .exec(&mut db)
        .await?;
    for p in loaded_user_include_posts_with_event.unwrap().posts.get() {
        println!("Got post with event {:?}", p);
        // This prints: Got post with event Post { id: 1, title: "My First Post", user_id: 1, user: <not loaded>, event_id: 1, event: Event { id: 1, name: "Conference 2024" }, product_id: 1, product: <not loaded> }

        // Here I can print the event name
        println!("Got event {:?}", p.event.get().name);
    }

    let loaded_user_include_posts_with_event_and_product = User::filter_by_id(user.id)
        .include(User::fields().posts())
        .include(User::fields().posts().event()) // will not be preloaded
        .include(User::fields().posts().product()) // will be preloaded
        .first()
        .exec(&mut db)
        .await?;
    for p in loaded_user_include_posts_with_event_and_product.unwrap().posts.get() {
        println!("Got post with event AND product {:?}", p);
        // This prints: Got post with event AND product Post { id: 1, title: "My First Post", user_id: 1, user: <not loaded>, event_id: 1, event: <not loaded>, product_id: 1, product: Product { id: 1, name: "Widget" } }
        // This time around, event is not preloaded, only product

        // Here trying to do something with the event will panic
        println!("Got event {:?}", p.event.get().name);

    }

    Ok(())
}
