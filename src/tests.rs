fn parse_double(e:&str) -> f64 {
    e.parse::<f64>().unwrap()
}

fn parse_double_line<'a, I1>(l:I1) -> std::iter::Map<I1, fn(&'a str) -> f64> where
    I1:Iterator<Item=&'a str> {
    l.map(parse_double)
}

fn trim_line(line: &str) -> std::str::SplitWhitespace {line.trim().split_whitespace()}






fn read_output(output: &str) -> (std::vec::Vec<[f64;2]>,std::vec::Vec<std::vec::Vec<i32>>,std::vec::Vec<[i32;2]>,std::vec::Vec<std::vec::Vec<i32>>) {


    let mut read_output = output.trim().lines().map(trim_line);

    let mut line1 = read_output.nth(1).unwrap();
    let nvertex = line1.next().and_then(|e| e.parse::<usize>().ok()).unwrap();
    let nregion = line1.next().and_then(|e| e.parse::<usize>().ok()).unwrap();



    let mut vertices = read_output.by_ref().skip(1).take(nvertex-1).map(parse_double_line).map(|mut e| [e.next().unwrap(), e.next().unwrap()]).collect::<Vec<[f64;2]>>();

    let mut regions = read_output.by_ref().take(nregion).map(|l| l.skip(1).map(|e| e.parse::<i32>().unwrap() - 1)).map(|e| {let mut l = e.collect::<Vec<i32>>();l.sort_by(|a, b| a.partial_cmp(b).unwrap());l}).collect::<Vec<Vec<i32>>>();


    let mut ridges:(std::vec::Vec<[i32;2]>,std::vec::Vec<std::vec::Vec<i32>>)= read_output.by_ref().skip(1).map(|l| {let mut base = l.map(|e| e.parse::<i32>().unwrap());let mut left = base.by_ref().skip(1).take(2);let mut l:[i32;2] = [left.next().unwrap(), left.next().unwrap()];l.sort();;let right = base.by_ref().map(|x| x - 1).collect::<Vec<i32>>();(l, right)}).unzip();

    let (mut ridge_points, mut ridge_vertices) = ridges;
    vertices.sort_by(|a, b| a.partial_cmp(b).unwrap());



    regions.sort_by(|a, b| a.partial_cmp(b).unwrap());


    ridge_points.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ridge_vertices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (vertices, regions, ridge_points, ridge_vertices)

}

#[test]
fn simple() {
    let output = "
    2
    5 10 1
    -10.101 -10.101
       0.5    0.5
       1.5    0.5
       0.5    1.5
       1.5    1.5
    2 0 1
    3 3 0 1
    2 0 3
    3 2 0 1
    4 4 3 1 2
    3 4 0 3
    2 0 2
    3 4 0 2
    2 0 4
    0
    12
    4 0 3 0 1
    4 0 1 0 1
    4 1 4 1 3
    4 1 2 0 3
    4 2 5 0 3
    4 3 4 1 2
    4 3 6 0 2
    4 4 5 3 4
    4 4 7 2 4
    4 5 8 0 4
    4 6 7 0 2
    4 7 8 0 4";




    let mut points = array![[0.0, 0.0], [0.0, 1.0], [0.0, 2.0],
                                     [1.0, 0.0], [1.0, 1.0], [1.0, 2.0],
                                     [2.0, 0.0], [2.0, 1.0], [2.0, 2.0]];

    println!("{:?}",points.shape());


    let (vertices, regions, ridge_points, ridge_vertices) = read_output(output);


    let voronoi = super::Voronoi::new(points).unwrap();

    let mut vvertices = voronoi.vertices.clone();
    vvertices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut vpoints = voronoi.ridge_points.clone().iter().map(|ps:&[i32;2]| {let mut temp = ps.clone(); temp.sort();temp} ).collect::<Vec<[i32;2]>>();

    vpoints.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut vvridgever:Vec<Vec<i32>> = voronoi.ridge_vertices.clone();

    vvridgever.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut vregions = voronoi.regions.clone().iter().map(|v| {let mut temp = v.clone();temp.sort();temp}).collect::<Vec<Vec<i32>>>();

    vregions.sort_by(|a, b| a.partial_cmp(b).unwrap());


    assert_eq!(vvertices, vertices, "vertices");
    assert_eq!(vregions, regions, "regions");
    assert_eq!(vvridgever, ridge_vertices, "ridge_vertices");
    assert_eq!(vpoints, ridge_points, "ridge_points");
}
