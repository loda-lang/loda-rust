use simple_pagerank::Pagerank;

pub fn compute_program_rank() {
    let mut pr = Pagerank::<&str>::new();
    pr.add_edge("source", "target");
    pr.add_edge("source", "another target");
    pr.calculate();

    // print result (always sorted)

    pr.nodes()
        .iter()
        .map(|(node, score)| println!("page {} with score {}", node, score))
        .for_each(drop);
}
