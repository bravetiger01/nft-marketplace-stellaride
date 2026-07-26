import Navbar from "@/components/Navbar";
import Contract from "@/components/Contract";

export default function Home() {
  return (
    <div className="min-h-screen bg-black text-white">
      <Navbar />
      <Contract />
    </div>
  );
}
