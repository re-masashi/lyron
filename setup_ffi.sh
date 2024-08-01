sudo cp ./lyvalue.cpp /usr/include/
echo "====COPIED INCLUDE FILE==="
echo "====BUILDING liblyron FILE==="
g++ -fPIC -c -Wall lyvalue.cpp -fpermissive
echo "====BUILT liblyron FILE==="
echo "====LINKING liblyron FILE==="
ld -shared lyvalue.o -o liblyron.so
echo "====LINKED liblyron FILE==="
echo "====liblyron.so successfully created==="
sudo mv liblyron.so /usr/lib/
echo "====liblyron.so placed in global lib directory==="
echo "cleaning up..."
rm lyvalue.o
echo "all done"